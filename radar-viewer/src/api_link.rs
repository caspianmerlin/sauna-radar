use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, mpsc::{Sender, Receiver, self}}, thread::JoinHandle, net::{TcpStream, Shutdown}, time::Duration, io::{BufWriter, Write}, fmt::format};
use std::thread;
use common::{aircraft_data::{Autopilot, fms_graphics::{FmsGraphic, FmsLine, FmsArc, FmsArcState}}, position::Position, ipc::radar_to_ui::PacketType, api_requests::ApiRequestType};
use serde::{Deserialize, Serialize};
use common::{ipc::{radar_to_ui, ui_to_radar}, aircraft_data::AircraftUpdate};
use log::{info, error};
use ureq::serde_json;

const API_POLL_INTERVAL: Duration = Duration::from_millis(100);
const API_REQUEST_TIMEOUT: Duration = Duration::from_millis(1000);
const AIRCRAFT_DATA_ENDPOINT: &str = "/api/aircraft/getAllWithFms";
const LOG_BUFFER_ENDPOINT: &str = "/api/commands/commandBuffer";
const TEXT_COMMAND_ENDPOINT: &str = "/api/commands/send/textCommand";


pub struct ApiLink {
    thread_should_terminate: Arc<AtomicBool>,
    rta_tx: Sender<radar_to_ui::PacketType>,
    msg_rx: Receiver<ImplMessage>,
    thread: Option<JoinHandle<()>>,
}

const INPUT: &str = r#"
[
  {
    "callsign": "EAG37BD",
    "delayMs": -1,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "AT72",
      "filedTas": 420,
      "origin": "EGBB",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 16000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "LUVUM DCT NANTI Y53 WAL M146 ROBOP M147 MAGEE"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9557671817687751,
          "degrees": 54.76142571246381,
          "percentage": 141.5566115234927,
          "degMinSec": {}
        },
        "radians": 0.9557671817687751,
        "degrees": 54.76142571246381,
        "degMinSec": {},
        "vrc": "N054.45.041.133",
        "nats": "544541N"
      },
      "longitude": {
        "angle": {
          "radians": -0.09542326884668917,
          "degrees": -5.46735057225748,
          "percentage": -9.571395635948376,
          "degMinSec": {}
        },
        "radians": -0.09542326884668917,
        "degrees": -5.46735057225748,
        "degMinSec": {},
        "vrc": "E005.28.002.462",
        "nats": "0052802W"
      },
      "indicatedAltitude": {
        "meters": 3047.8675993589727,
        "feet": 9999.565934680892,
        "nauticalMiles": 1.6457168463061407,
        "statuteMiles": 1.8938571239952258
      },
      "trueAltitude": {
        "meters": 3171.3958956089728,
        "feet": 10404.842510149741,
        "nauticalMiles": 1.7124167902856224,
        "statuteMiles": 1.9706140487111348
      },
      "pressureAltitude": {
        "meters": 3047.8675993589727,
        "feet": 9999.565934680892,
        "nauticalMiles": 1.6457168463061407,
        "statuteMiles": 1.8938571239952258
      },
      "densityAltitude": {
        "meters": 3309.989647284798,
        "feet": 10859.546434397857,
        "nauticalMiles": 1.7872514294194375,
        "statuteMiles": 2.0567322134265873
      },
      "heading_Mag": {
        "angle": {
          "radians": 4.699115752863053,
          "degrees": 269.2395000824933,
          "percentage": 7533.520174280019,
          "degMinSec": {}
        },
        "radians": 4.699115752863053,
        "degrees": 269.2395000824933
      },
      "heading_True": {
        "angle": {
          "radians": 4.6759583558689854,
          "degrees": 267.9126789702243,
          "percentage": 2743.7288746458667,
          "degMinSec": {}
        },
        "radians": 4.6759583558689854,
        "degrees": 267.9126789702243
      },
      "track_True": {
        "angle": {
          "radians": 4.765213158982256,
          "degrees": 273.0266024898858,
          "percentage": -1891.3113694676447,
          "degMinSec": {}
        },
        "radians": 4.765213158982256,
        "degrees": 273.0266024898858
      },
      "track_Mag": {
        "angle": {
          "radians": 4.788370555976325,
          "degrees": 274.3534236021549,
          "percentage": -1313.5748386563446,
          "degMinSec": {}
        },
        "radians": 4.788370555976325,
        "degrees": 274.3534236021549
      },
      "bank": {
        "radians": 0.4363323129985582,
        "degrees": 24.999999999998614,
        "percentage": 46.63076581549691,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.055519092886883926,
        "degrees": 3.1810097048132384,
        "percentage": 5.557620676244792,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.60924911295186,
        "knots": 249.9963172327168,
        "feetPerMinute": 25316.78213158422
      },
      "trueAirSpeed": {
        "metersPerSecond": 150.55659137944318,
        "knots": 292.6585268133823,
        "feetPerMinute": 29637.12523567994
      },
      "groundSpeed": {
        "metersPerSecond": 150.3065357663676,
        "knots": 292.17245771023903,
        "feetPerMinute": 29587.901688223767
      },
      "machNumber": 0.4515390337676199,
      "verticalSpeed": {
        "metersPerSecond": 0.011832163774833841,
        "knots": 0.02299988056072811,
        "feetPerMinute": 2.3291661719415515
      },
      "flightPathAngle": {
        "radians": 7.872022124696312E-05,
        "degrees": 0.004510336439787058,
        "percentage": 0.007872022140956954,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 43.26247075208207,
        "knots": 84.09549419661022,
        "feetPerMinute": 8516.234672535657
      },
      "velocity_Y": {
        "metersPerSecond": 0.011832163774833841,
        "knots": 0.02299988056072811,
        "feetPerMinute": 2.3291661719415515
      },
      "velocity_Z": {
        "metersPerSecond": -143.94586940413257,
        "knots": -279.80831456600663,
        "feetPerMinute": -28335.801970551256
      },
      "heading_Velocity": {
        "radiansPerSecond": 0.030440492699897712,
        "degreesPerSecond": 1.7441117580029313
      },
      "bank_Velocity": {
        "radiansPerSecond": 8.060219158778636E-14,
        "degreesPerSecond": 4.618165397485026E-12
      },
      "pitch_Velocity": {
        "radiansPerSecond": 2.909172328580123E-05,
        "degreesPerSecond": 0.00166683296303887
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.00421065718903868,
        "knotsPerSecond": 0.008184860712969705
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102675.921875,
        "hectopascals": 1026.75921875,
        "inchesOfMercury": 30.323662691966923
      },
      "windDirection": {
        "angle": {
          "radians": 3.123718696229405,
          "degrees": 178.97589768005298,
          "percentage": -1.7875861051142534,
          "degMinSec": {}
        },
        "radians": 3.123718696229405,
        "degrees": 178.97589768005298
      },
      "windSpeed": {
        "metersPerSecond": 13.476017145921903,
        "knots": 26.195275072997415,
        "feetPerMinute": 2652.759365581585
      },
      "windXComp": {
        "metersPerSecond": 13.473696979951363,
        "knots": 26.190765032296575,
        "feetPerMinute": 2652.302639982218
      },
      "windHComp": {
        "metersPerSecond": 0.25005561307558644,
        "knots": 0.4860691031433002,
        "feetPerMinute": 49.22354745617442
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9555677654668955,
            "degrees": 54.75000000000001,
            "percentage": 141.49672721156952,
            "degMinSec": {}
          },
          "radians": 0.9555677654668955,
          "degrees": 54.75000000000001,
          "degMinSec": {},
          "vrc": "N054.45.000.000",
          "nats": "544500N"
        },
        "lon": {
          "angle": {
            "radians": -0.09599310885968748,
            "degrees": -5.499999999999964,
            "percentage": -9.628904819753796,
            "degMinSec": {}
          },
          "radians": -0.09599310885968748,
          "degrees": -5.499999999999964,
          "degMinSec": {},
          "vrc": "E005.29.060.000",
          "nats": "0053000W"
        },
        "geoPotentialHeight": {
          "meters": 3161.80126953125,
          "feet": 10373.364077128907,
          "nauticalMiles": 1.7072361066583424,
          "statuteMiles": 1.9646522244661488
        },
        "levelPressure": {
          "pascals": 70000,
          "hectopascals": 700,
          "inchesOfMercury": 20.67336089781453
        },
        "temp": {
          "kelvin": 276.6999816894531,
          "celsius": 3.5499816894531477
        },
        "v": {
          "metersPerSecond": 13.473864555358887,
          "knots": 26.19109077274704,
          "feetPerMinute": 2652.335627268219
        },
        "u": {
          "metersPerSecond": -0.24085693061351776,
          "knots": -0.4681882994315028,
          "feetPerMinute": -47.41278313404322
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102675.921875,
          "hectopascals": 1026.75921875,
          "inchesOfMercury": 30.323662691966923
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9557671817687751,
            "degrees": 54.76142571246381,
            "percentage": 141.5566115234927,
            "degMinSec": {}
          },
          "radians": 0.9557671817687751,
          "degrees": 54.76142571246381,
          "degMinSec": {},
          "vrc": "N054.45.041.133",
          "nats": "544541N"
        },
        "lon": {
          "angle": {
            "radians": -0.09542326884668917,
            "degrees": -5.46735057225748,
            "percentage": -9.571395635948376,
            "degMinSec": {}
          },
          "radians": -0.09542326884668917,
          "degrees": -5.46735057225748,
          "degMinSec": {},
          "vrc": "E005.28.002.462",
          "nats": "0052802W"
        },
        "alt": {
          "meters": 3171.3958956089728,
          "feet": 10404.842510149741,
          "nauticalMiles": 1.7124167902856224,
          "statuteMiles": 1.9706140487111348
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 8.060219158778636E-14,
        "degreesPerSecond": 4.618165397485026E-12
      },
      "pitchRate": {
        "radiansPerSecond": 2.909172328580123E-05,
        "degreesPerSecond": 0.00166683296303887
      },
      "yawRate": {
        "radiansPerSecond": 0.030440492699897712,
        "degreesPerSecond": 1.7441117580029313
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 32.604433777994345,
      "thrustLeverVel": 0.08051394258365008,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 16000,
      "departureAirport": {
        "identifier": "EGBB",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9154930666057738,
              "degrees": 52.45388889,
              "percentage": 130.1056003377022,
              "degMinSec": {}
            },
            "radians": 0.9154930666057738,
            "degrees": 52.45388889,
            "degMinSec": {},
            "vrc": "N052.27.014.000",
            "nats": "522714N"
          },
          "lon": {
            "angle": {
              "radians": -0.030509325029793644,
              "degrees": -1.7480555600000203,
              "percentage": -3.0518794774584364,
              "degMinSec": {}
            },
            "radians": -0.030509325029793644,
            "degrees": -1.7480555600000203,
            "degMinSec": {},
            "vrc": "E001.44.053.000",
            "nats": "0014453W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "instr": {
          "outboundTurnLeg": {
            "startPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.956292707344866,
                      "degrees": 54.79153611,
                      "percentage": 141.71458787308552,
                      "degMinSec": {}
                    },
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "degMinSec": {},
                    "vrc": "N054.47.029.530",
                    "nats": "544730N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09788397919814429,
                      "degrees": -5.608338890000012,
                      "percentage": -9.819779969455576,
                      "degMinSec": {}
                    },
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "degMinSec": {},
                    "vrc": "E005.36.030.020",
                    "nats": "0053630W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": "MAGEE"
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "endPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.9543867220711226,
                      "degrees": 54.68233119800042,
                      "percentage": 141.14275372324715,
                      "degMinSec": {}
                    },
                    "radians": 0.9543867220711226,
                    "degrees": 54.68233119800042,
                    "degMinSec": {},
                    "vrc": "N054.40.056.392",
                    "nats": "544056N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09678698185341439,
                      "degrees": -5.54548557200993,
                      "percentage": -9.70903430692095,
                      "degMinSec": {}
                    },
                    "radians": -0.09678698185341439,
                    "degrees": -5.54548557200993,
                    "degMinSec": {},
                    "vrc": "E005.32.043.748",
                    "nats": "0053244W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": "MAGEEH_OS161.59344306463097Â°12795.859963705923m"
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "legLength": {
              "meters": -1,
              "feet": -3.28084,
              "nauticalMiles": -0.0005399568034557236,
              "statuteMiles": -0.0006213711922373339
            },
            "initialTrueCourse": {
              "angle": {
                "radians": 4.391133957906703,
                "degrees": 251.59344306463097,
                "percentage": 300.49626971172285,
                "degMinSec": {}
              },
              "radians": 4.391133957906703,
              "degrees": 251.59344306463097
            },
            "finalTrueCourse": {
              "angle": {
                "radians": 1.2495413043169101,
                "degrees": 71.59344306463099,
                "percentage": 300.49626971172296,
                "degMinSec": {}
              },
              "radians": 1.2495413043169101,
              "degrees": 71.59344306463099
            },
            "legType": "RADIUS_TO_FIX",
            "arcInfo": {
              "center": {
                "lat": {
                  "angle": {
                    "radians": 0.9553395019776336,
                    "degrees": 54.73692146544837,
                    "percentage": 141.42822161542986,
                    "degMinSec": {}
                  },
                  "radians": 0.9553395019776336,
                  "degrees": 54.73692146544837,
                  "degMinSec": {},
                  "vrc": "N054.44.012.917",
                  "nats": "544413N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09733621979321683,
                    "degrees": -5.5769545879090705,
                    "percentage": -9.76447880414375,
                    "degMinSec": {}
                  },
                  "radians": -0.09733621979321683,
                  "degrees": -5.5769545879090705,
                  "degMinSec": {},
                  "vrc": "E005.34.037.037",
                  "nats": "0053437W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "tangentialPointA": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointARadial": {
                "angle": {
                  "radians": 5.963273395990143,
                  "degrees": 341.6703976728809,
                  "percentage": -33.129164544758346,
                  "degMinSec": {}
                },
                "radians": 5.963273395990143,
                "degrees": 341.6703976728809
              },
              "tangentialPointB": {
                "lat": {
                  "angle": {
                    "radians": 0.9543867220711226,
                    "degrees": 54.68233119800042,
                    "percentage": 141.14275372324715,
                    "degMinSec": {}
                  },
                  "radians": 0.9543867220711226,
                  "degrees": 54.68233119800042,
                  "degMinSec": {},
                  "vrc": "N054.40.056.392",
                  "nats": "544056N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09678698185341439,
                    "degrees": -5.54548557200993,
                    "percentage": -9.70903430692095,
                    "degMinSec": {}
                  },
                  "radians": -0.09678698185341439,
                  "degrees": -5.54548557200993,
                  "degMinSec": {},
                  "vrc": "E005.32.043.748",
                  "nats": "0053244W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointBRadial": {
                "angle": {
                  "radians": 2.819889324011255,
                  "degrees": 161.56775695984362,
                  "percentage": -33.32808621391597,
                  "degMinSec": {}
                },
                "radians": 2.819889324011255,
                "degrees": 161.56775695984362
              },
              "radiusM": {
                "meters": 6397.93254836296,
                "feet": 20990.593021971134,
                "nauticalMiles": 3.454607207539395,
                "statuteMiles": 3.9754909754303367
              },
              "bisectorIntersection": null
            },
            "uiLines": [
              {
                "startPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.956292707344866,
                      "degrees": 54.79153611,
                      "percentage": 141.71458787308552,
                      "degMinSec": {}
                    },
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "degMinSec": {},
                    "vrc": "N054.47.029.530",
                    "nats": "544730N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09788397919814429,
                      "degrees": -5.608338890000012,
                      "percentage": -9.819779969455576,
                      "degMinSec": {}
                    },
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "degMinSec": {},
                    "vrc": "E005.36.030.020",
                    "nats": "0053630W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "endPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.9543867220711226,
                      "degrees": 54.68233119800042,
                      "percentage": 141.14275372324715,
                      "degMinSec": {}
                    },
                    "radians": 0.9543867220711226,
                    "degrees": 54.68233119800042,
                    "degMinSec": {},
                    "vrc": "N054.40.056.392",
                    "nats": "544056N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09678698185341439,
                      "degrees": -5.54548557200993,
                      "percentage": -9.70903430692095,
                      "degMinSec": {}
                    },
                    "radians": -0.09678698185341439,
                    "degrees": -5.54548557200993,
                    "degMinSec": {},
                    "vrc": "E005.32.043.748",
                    "nats": "0053244W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                }
              }
            ]
          },
          "outboundLeg": {
            "legType": "TRACK_TO_FIX",
            "initialTrueCourse": {
              "angle": {
                "radians": 1.2495413043168915,
                "degrees": 71.59344306462991,
                "percentage": 300.49626971170426,
                "degMinSec": {}
              },
              "radians": 1.2495413043168915,
              "degrees": 71.59344306462991
            },
            "finalTrueCourse": {
              "angle": {
                "radians": 1.2502677831257607,
                "degrees": 71.63506723428381,
                "percentage": 301.22650775492684,
                "degMinSec": {}
              },
              "radians": 1.2502677831257607,
              "degrees": 71.63506723428381
            },
            "endPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.9577312372084053,
                      "degrees": 54.87395779988433,
                      "percentage": 142.14822547879896,
                      "degMinSec": {}
                    },
                    "radians": 0.9577312372084053,
                    "degrees": 54.87395779988433,
                    "degMinSec": {},
                    "vrc": "N054.52.026.248",
                    "nats": "545226N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09604986708678709,
                      "degrees": -5.503252006865416,
                      "percentage": -9.634633297626038,
                      "degMinSec": {}
                    },
                    "radians": -0.09604986708678709,
                    "degrees": -5.503252006865416,
                    "degMinSec": {},
                    "vrc": "E005.30.011.707",
                    "nats": "0053012W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": ""
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "startPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.9575613369492182,
                      "degrees": 54.86422323209474,
                      "percentage": 142.0969175968984,
                      "degMinSec": {}
                    },
                    "radians": 0.9575613369492182,
                    "degrees": 54.86422323209474,
                    "degMinSec": {},
                    "vrc": "N054.51.051.204",
                    "nats": "545151N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09693815736892653,
                      "degrees": -5.55414729101449,
                      "percentage": -9.724294588698527,
                      "degMinSec": {}
                    },
                    "radians": -0.09693815736892653,
                    "degrees": -5.55414729101449,
                    "degMinSec": {},
                    "vrc": "E005.33.014.930",
                    "nats": "0053315W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": "MAGEEH_OS23.219459675593278Â°8796.263889786274m"
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "legLength": {
              "meters": 3431.8009608859725,
              "feet": 11259.189864513133,
              "nauticalMiles": 1.8530242769362701,
              "statuteMiles": 2.132422254586945
            },
            "uiLines": [
              {
                "startPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.9575613369492182,
                      "degrees": 54.86422323209474,
                      "percentage": 142.0969175968984,
                      "degMinSec": {}
                    },
                    "radians": 0.9575613369492182,
                    "degrees": 54.86422323209474,
                    "degMinSec": {},
                    "vrc": "N054.51.051.204",
                    "nats": "545151N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09693815736892653,
                      "degrees": -5.55414729101449,
                      "percentage": -9.724294588698527,
                      "degMinSec": {}
                    },
                    "radians": -0.09693815736892653,
                    "degrees": -5.55414729101449,
                    "degMinSec": {},
                    "vrc": "E005.33.014.930",
                    "nats": "0053315W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "endPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.9577312372084053,
                      "degrees": 54.87395779988433,
                      "percentage": 142.14822547879896,
                      "degMinSec": {}
                    },
                    "radians": 0.9577312372084053,
                    "degrees": 54.87395779988433,
                    "degMinSec": {},
                    "vrc": "N054.52.026.248",
                    "nats": "545226N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09604986708678709,
                      "degrees": -5.503252006865416,
                      "percentage": -9.634633297626038,
                      "degMinSec": {}
                    },
                    "radians": -0.09604986708678709,
                    "degrees": -5.503252006865416,
                    "degMinSec": {},
                    "vrc": "E005.30.011.707",
                    "nats": "0053012W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                }
              }
            ]
          },
          "inboundTurnLeg": {
            "startPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.9577312372084053,
                      "degrees": 54.87395779988433,
                      "percentage": 142.14822547879896,
                      "degMinSec": {}
                    },
                    "radians": 0.9577312372084053,
                    "degrees": 54.87395779988433,
                    "degMinSec": {},
                    "vrc": "N054.52.026.248",
                    "nats": "545226N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09604986708678709,
                      "degrees": -5.503252006865416,
                      "percentage": -9.634633297626038,
                      "degMinSec": {}
                    },
                    "radians": -0.09604986708678709,
                    "degrees": -5.503252006865416,
                    "degMinSec": {},
                    "vrc": "E005.30.011.707",
                    "nats": "0053012W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": ""
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "endPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.9558026586935596,
                      "degrees": 54.76345839052407,
                      "percentage": 141.56726871466853,
                      "degMinSec": {}
                    },
                    "radians": 0.9558026586935596,
                    "degrees": 54.76345839052407,
                    "degMinSec": {},
                    "vrc": "N054.45.048.450",
                    "nats": "544548N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09494034016362995,
                      "degrees": -5.439680796912376,
                      "percentage": -9.522662597678853,
                      "degMinSec": {}
                    },
                    "radians": -0.09494034016362995,
                    "degrees": -5.439680796912376,
                    "degMinSec": {},
                    "vrc": "E005.26.022.851",
                    "nats": "0052623W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": "MAGEEH_IS1161.6350672342838Â°12944.394724191601m"
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "legLength": {
              "meters": -1,
              "feet": -3.28084,
              "nauticalMiles": -0.0005399568034557236,
              "statuteMiles": -0.0006213711922373339
            },
            "initialTrueCourse": {
              "angle": {
                "radians": 1.2502677831257607,
                "degrees": 71.63506723428381,
                "percentage": 301.22650775492684,
                "degMinSec": {}
              },
              "radians": 1.2502677831257607,
              "degrees": 71.63506723428381
            },
            "finalTrueCourse": {
              "angle": {
                "radians": 4.393093073358551,
                "degrees": 251.7056921116007,
                "percentage": 302.47286235229103,
                "degMinSec": {}
              },
              "radians": 4.393093073358551,
              "degrees": 251.7056921116007
            },
            "legType": "RADIUS_TO_FIX",
            "arcInfo": {
              "center": {
                "lat": {
                  "angle": {
                    "radians": 0.9567671245480275,
                    "degrees": 54.81871821346956,
                    "percentage": 141.85740294405875,
                    "degMinSec": {}
                  },
                  "radians": 0.9567671245480275,
                  "degrees": 54.81871821346956,
                  "degMinSec": {},
                  "vrc": "N054.49.007.386",
                  "nats": "544907N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09549379951414849,
                    "degrees": -5.4713916818291395,
                    "percentage": -9.578513365039377,
                    "degMinSec": {}
                  },
                  "radians": -0.09549379951414849,
                  "degrees": -5.4713916818291395,
                  "degMinSec": {},
                  "vrc": "E005.28.017.010",
                  "nats": "0052817W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "tangentialPointA": {
                "lat": {
                  "angle": {
                    "radians": 0.9577312372084053,
                    "degrees": 54.87395779988433,
                    "percentage": 142.14822547879896,
                    "degMinSec": {}
                  },
                  "radians": 0.9577312372084053,
                  "degrees": 54.87395779988433,
                  "degMinSec": {},
                  "vrc": "N054.52.026.248",
                  "nats": "545226N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09604986708678709,
                    "degrees": -5.503252006865416,
                    "percentage": -9.634633297626038,
                    "degMinSec": {}
                  },
                  "radians": -0.09604986708678709,
                  "degrees": -5.503252006865416,
                  "degMinSec": {},
                  "vrc": "E005.30.011.707",
                  "nats": "0053012W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointARadial": {
                "angle": {
                  "radians": 5.962785627287146,
                  "degrees": 341.6424505848206,
                  "percentage": -33.18330363203808,
                  "degMinSec": {}
                },
                "radians": 5.962785627287146,
                "degrees": 341.6424505848206
              },
              "tangentialPointB": {
                "lat": {
                  "angle": {
                    "radians": 0.9558026586935596,
                    "degrees": 54.76345839052407,
                    "percentage": 141.56726871466853,
                    "degMinSec": {}
                  },
                  "radians": 0.9558026586935596,
                  "degrees": 54.76345839052407,
                  "degMinSec": {},
                  "vrc": "N054.45.048.450",
                  "nats": "544548N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09494034016362995,
                    "degrees": -5.439680796912376,
                    "percentage": -9.522662597678853,
                    "degMinSec": {}
                  },
                  "radians": -0.09494034016362995,
                  "degrees": -5.439680796912376,
                  "degMinSec": {},
                  "vrc": "E005.26.022.851",
                  "nats": "0052623W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointBRadial": {
                "angle": {
                  "radians": 2.821844539650172,
                  "degrees": 161.67978256399155,
                  "percentage": -33.11098806309035,
                  "degMinSec": {}
                },
                "radians": 2.821844539650172,
                "degrees": 161.67978256399155
              },
              "radiusM": {
                "meters": 6472.197705558189,
                "feet": 21234.24512030353,
                "nauticalMiles": 3.494707184426668,
                "statuteMiles": 4.021637204698429
              },
              "bisectorIntersection": null
            },
            "uiLines": [
              {
                "startPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.9577312372084053,
                      "degrees": 54.87395779988433,
                      "percentage": 142.14822547879896,
                      "degMinSec": {}
                    },
                    "radians": 0.9577312372084053,
                    "degrees": 54.87395779988433,
                    "degMinSec": {},
                    "vrc": "N054.52.026.248",
                    "nats": "545226N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09604986708678709,
                      "degrees": -5.503252006865416,
                      "percentage": -9.634633297626038,
                      "degMinSec": {}
                    },
                    "radians": -0.09604986708678709,
                    "degrees": -5.503252006865416,
                    "degMinSec": {},
                    "vrc": "E005.30.011.707",
                    "nats": "0053012W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "endPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.9558026586935596,
                      "degrees": 54.76345839052407,
                      "percentage": 141.56726871466853,
                      "degMinSec": {}
                    },
                    "radians": 0.9558026586935596,
                    "degrees": 54.76345839052407,
                    "degMinSec": {},
                    "vrc": "N054.45.048.450",
                    "nats": "544548N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09494034016362995,
                      "degrees": -5.439680796912376,
                      "percentage": -9.522662597678853,
                      "degMinSec": {}
                    },
                    "radians": -0.09494034016362995,
                    "degrees": -5.439680796912376,
                    "degMinSec": {},
                    "vrc": "E005.26.022.851",
                    "nats": "0052623W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                }
              }
            ]
          },
          "inboundLeg": {
            "legType": "TRACK_TO_FIX",
            "initialTrueCourse": {
              "angle": {
                "radians": 4.393093073358551,
                "degrees": 251.7056921116007,
                "percentage": 302.47286235229103,
                "degMinSec": {}
              },
              "radians": 4.393093073358551,
              "degrees": 251.7056921116007
            },
            "finalTrueCourse": {
              "angle": {
                "radians": 4.391133957906843,
                "degrees": 251.59344306463896,
                "percentage": 300.4962697118627,
                "degMinSec": {}
              },
              "radians": 4.391133957906843,
              "degrees": 251.59344306463896
            },
            "endPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.956292707344866,
                      "degrees": 54.79153611,
                      "percentage": 141.71458787308552,
                      "degMinSec": {}
                    },
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "degMinSec": {},
                    "vrc": "N054.47.029.530",
                    "nats": "544730N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09788397919814429,
                      "degrees": -5.608338890000012,
                      "percentage": -9.819779969455576,
                      "degMinSec": {}
                    },
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "degMinSec": {},
                    "vrc": "E005.36.030.020",
                    "nats": "0053630W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": "MAGEE"
              },
              "pointType": "FLY_OVER",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "startPoint": {
              "point": {
                "pointPosition": {
                  "lat": {
                    "angle": {
                      "radians": 0.9567510322293694,
                      "degrees": 54.81779619152787,
                      "percentage": 141.85255548144173,
                      "degMinSec": {}
                    },
                    "radians": 0.9567510322293694,
                    "degrees": 54.81779619152787,
                    "degMinSec": {},
                    "vrc": "N054.49.004.066",
                    "nats": "544904N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.0954866044417777,
                      "degrees": -5.470979434549002,
                      "percentage": -9.577787256973586,
                      "degMinSec": {}
                    },
                    "radians": -0.0954866044417777,
                    "degrees": -5.470979434549002,
                    "degMinSec": {},
                    "vrc": "E005.28.015.526",
                    "nats": "0052816W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "pointName": ""
              },
              "pointType": "FLY_BY",
              "lowerAltitudeConstraint": 0,
              "upperAltitudeConstraint": 0,
              "angleConstraint": -1,
              "vnavTargetAltitude": -1,
              "speedConstraintType": "FREE",
              "speedConstraint": 0
            },
            "legLength": {
              "meters": 9274.860239074635,
              "feet": 30429.332466765623,
              "nauticalMiles": 5.008023887189328,
              "statuteMiles": 5.76313096458845
            },
            "uiLines": [
              {
                "startPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.9567510322293694,
                      "degrees": 54.81779619152787,
                      "percentage": 141.85255548144173,
                      "degMinSec": {}
                    },
                    "radians": 0.9567510322293694,
                    "degrees": 54.81779619152787,
                    "degMinSec": {},
                    "vrc": "N054.49.004.066",
                    "nats": "544904N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.0954866044417777,
                      "degrees": -5.470979434549002,
                      "percentage": -9.577787256973586,
                      "degMinSec": {}
                    },
                    "radians": -0.0954866044417777,
                    "degrees": -5.470979434549002,
                    "degMinSec": {},
                    "vrc": "E005.28.015.526",
                    "nats": "0052816W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                },
                "endPoint": {
                  "lat": {
                    "angle": {
                      "radians": 0.956292707344866,
                      "degrees": 54.79153611,
                      "percentage": 141.71458787308552,
                      "degMinSec": {}
                    },
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "degMinSec": {},
                    "vrc": "N054.47.029.530",
                    "nats": "544730N"
                  },
                  "lon": {
                    "angle": {
                      "radians": -0.09788397919814429,
                      "degrees": -5.608338890000012,
                      "percentage": -9.819779969455576,
                      "degMinSec": {}
                    },
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "degMinSec": {},
                    "vrc": "E005.36.030.020",
                    "nats": "0053630W"
                  },
                  "alt": {
                    "meters": 0,
                    "feet": 0,
                    "nauticalMiles": 0,
                    "statuteMiles": 0
                  }
                }
              }
            ]
          },
          "alongTrack_M": null,
          "crossTrack_M": null,
          "currentTrueCourse": null,
          "radius_M": null,
          "holdPhase": "INBOUND",
          "magneticCourse": {
            "angle": {
              "radians": 4.415683007545653,
              "degrees": 252.99999999999994,
              "percentage": 327.08526184841264,
              "degMinSec": {}
            },
            "radians": 4.415683007545653,
            "degrees": 252.99999999999994
          },
          "trueCourse": {
            "angle": {
              "radians": 4.391133957906703,
              "degrees": 251.59344306463097,
              "percentage": 300.49626971172285,
              "degMinSec": {}
            },
            "radians": 4.391133957906703,
            "degrees": 251.59344306463097
          },
          "exitArmed": false
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "percentage": 141.71458787308552,
                  "degMinSec": {}
                },
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "degMinSec": {},
                "vrc": "N054.47.029.530",
                "nats": "544730N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "percentage": -9.819779969455576,
                  "degMinSec": {}
                },
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "degMinSec": {},
                "vrc": "E005.36.030.020",
                "nats": "0053630W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MAGEE"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "percentage": 141.71458787308552,
                  "degMinSec": {}
                },
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "degMinSec": {},
                "vrc": "N054.47.029.530",
                "nats": "544730N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "percentage": -9.819779969455576,
                  "degMinSec": {}
                },
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "degMinSec": {},
                "vrc": "E005.36.030.020",
                "nats": "0053630W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MAGEE"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 4.391133957906703,
            "degrees": 251.59344306463097,
            "percentage": 300.49626971172285,
            "degMinSec": {}
          },
          "radians": 4.391133957906703,
          "degrees": 251.59344306463097
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 4.391133957906703,
            "degrees": 251.59344306463097,
            "percentage": 300.49626971172285,
            "degMinSec": {}
          },
          "radians": 4.391133957906703,
          "degrees": 251.59344306463097
        },
        "legLength": {
          "meters": 0,
          "feet": 0,
          "nauticalMiles": 0,
          "statuteMiles": 0
        },
        "exitArmed": false,
        "legType": "HOLD_TO_MANUAL",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "percentage": 141.71458787308552,
                  "degMinSec": {}
                },
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "degMinSec": {},
                "vrc": "N054.47.029.530",
                "nats": "544730N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "percentage": -9.819779969455576,
                  "degMinSec": {}
                },
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "degMinSec": {},
                "vrc": "E005.36.030.020",
                "nats": "0053630W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9543867220711226,
                  "degrees": 54.68233119800042,
                  "percentage": 141.14275372324715,
                  "degMinSec": {}
                },
                "radians": 0.9543867220711226,
                "degrees": 54.68233119800042,
                "degMinSec": {},
                "vrc": "N054.40.056.392",
                "nats": "544056N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09678698185341439,
                  "degrees": -5.54548557200993,
                  "percentage": -9.70903430692095,
                  "degMinSec": {}
                },
                "radians": -0.09678698185341439,
                "degrees": -5.54548557200993,
                "degMinSec": {},
                "vrc": "E005.32.043.748",
                "nats": "0053244W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          },
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9575613369492182,
                  "degrees": 54.86422323209474,
                  "percentage": 142.0969175968984,
                  "degMinSec": {}
                },
                "radians": 0.9575613369492182,
                "degrees": 54.86422323209474,
                "degMinSec": {},
                "vrc": "N054.51.051.204",
                "nats": "545151N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09693815736892653,
                  "degrees": -5.55414729101449,
                  "percentage": -9.724294588698527,
                  "degMinSec": {}
                },
                "radians": -0.09693815736892653,
                "degrees": -5.55414729101449,
                "degMinSec": {},
                "vrc": "E005.33.014.930",
                "nats": "0053315W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9577312372084053,
                  "degrees": 54.87395779988433,
                  "percentage": 142.14822547879896,
                  "degMinSec": {}
                },
                "radians": 0.9577312372084053,
                "degrees": 54.87395779988433,
                "degMinSec": {},
                "vrc": "N054.52.026.248",
                "nats": "545226N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09604986708678709,
                  "degrees": -5.503252006865416,
                  "percentage": -9.634633297626038,
                  "degMinSec": {}
                },
                "radians": -0.09604986708678709,
                "degrees": -5.503252006865416,
                "degMinSec": {},
                "vrc": "E005.30.011.707",
                "nats": "0053012W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          },
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9577312372084053,
                  "degrees": 54.87395779988433,
                  "percentage": 142.14822547879896,
                  "degMinSec": {}
                },
                "radians": 0.9577312372084053,
                "degrees": 54.87395779988433,
                "degMinSec": {},
                "vrc": "N054.52.026.248",
                "nats": "545226N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09604986708678709,
                  "degrees": -5.503252006865416,
                  "percentage": -9.634633297626038,
                  "degMinSec": {}
                },
                "radians": -0.09604986708678709,
                "degrees": -5.503252006865416,
                "degMinSec": {},
                "vrc": "E005.30.011.707",
                "nats": "0053012W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9558026586935596,
                  "degrees": 54.76345839052407,
                  "percentage": 141.56726871466853,
                  "degMinSec": {}
                },
                "radians": 0.9558026586935596,
                "degrees": 54.76345839052407,
                "degMinSec": {},
                "vrc": "N054.45.048.450",
                "nats": "544548N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09494034016362995,
                  "degrees": -5.439680796912376,
                  "percentage": -9.522662597678853,
                  "degMinSec": {}
                },
                "radians": -0.09494034016362995,
                "degrees": -5.439680796912376,
                "degMinSec": {},
                "vrc": "E005.26.022.851",
                "nats": "0052623W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          },
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9567510322293694,
                  "degrees": 54.81779619152787,
                  "percentage": 141.85255548144173,
                  "degMinSec": {}
                },
                "radians": 0.9567510322293694,
                "degrees": 54.81779619152787,
                "degMinSec": {},
                "vrc": "N054.49.004.066",
                "nats": "544904N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0954866044417777,
                  "degrees": -5.470979434549002,
                  "percentage": -9.577787256973586,
                  "degMinSec": {}
                },
                "radians": -0.0954866044417777,
                "degrees": -5.470979434549002,
                "degMinSec": {},
                "vrc": "E005.28.015.526",
                "nats": "0052816W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "percentage": 141.71458787308552,
                  "degMinSec": {}
                },
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "degMinSec": {},
                "vrc": "N054.47.029.530",
                "nats": "544730N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "percentage": -9.819779969455576,
                  "degMinSec": {}
                },
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "degMinSec": {},
                "vrc": "E005.36.030.020",
                "nats": "0053630W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [],
      "asString": "MAGEE =(HM)=> INBOUND; ",
      "fmsLines": [
        {
          "center": {
            "lat": {
              "angle": {
                "radians": 0.9553395019776336,
                "degrees": 54.73692146544837,
                "percentage": 141.42822161542986,
                "degMinSec": {}
              },
              "radians": 0.9553395019776336,
              "degrees": 54.73692146544837,
              "degMinSec": {},
              "vrc": "N054.44.012.917",
              "nats": "544413N"
            },
            "lon": {
              "angle": {
                "radians": -0.09733621979321683,
                "degrees": -5.5769545879090705,
                "percentage": -9.76447880414375,
                "degMinSec": {}
              },
              "radians": -0.09733621979321683,
              "degrees": -5.5769545879090705,
              "degMinSec": {},
              "vrc": "E005.34.037.037",
              "nats": "0053437W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "radius_m": 6397.93254836296,
          "startTrueBearing": 341.6703976728809,
          "endTrueBearing": 161.56775695984362,
          "clockwise": false,
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9543867220711226,
                "degrees": 54.68233119800042,
                "percentage": 141.14275372324715,
                "degMinSec": {}
              },
              "radians": 0.9543867220711226,
              "degrees": 54.68233119800042,
              "degMinSec": {},
              "vrc": "N054.40.056.392",
              "nats": "544056N"
            },
            "lon": {
              "angle": {
                "radians": -0.09678698185341439,
                "degrees": -5.54548557200993,
                "percentage": -9.70903430692095,
                "degMinSec": {}
              },
              "radians": -0.09678698185341439,
              "degrees": -5.54548557200993,
              "degMinSec": {},
              "vrc": "E005.32.043.748",
              "nats": "0053244W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9575613369492182,
                "degrees": 54.86422323209474,
                "percentage": 142.0969175968984,
                "degMinSec": {}
              },
              "radians": 0.9575613369492182,
              "degrees": 54.86422323209474,
              "degMinSec": {},
              "vrc": "N054.51.051.204",
              "nats": "545151N"
            },
            "lon": {
              "angle": {
                "radians": -0.09693815736892653,
                "degrees": -5.55414729101449,
                "percentage": -9.724294588698527,
                "degMinSec": {}
              },
              "radians": -0.09693815736892653,
              "degrees": -5.55414729101449,
              "degMinSec": {},
              "vrc": "E005.33.014.930",
              "nats": "0053315W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9577312372084053,
                "degrees": 54.87395779988433,
                "percentage": 142.14822547879896,
                "degMinSec": {}
              },
              "radians": 0.9577312372084053,
              "degrees": 54.87395779988433,
              "degMinSec": {},
              "vrc": "N054.52.026.248",
              "nats": "545226N"
            },
            "lon": {
              "angle": {
                "radians": -0.09604986708678709,
                "degrees": -5.503252006865416,
                "percentage": -9.634633297626038,
                "degMinSec": {}
              },
              "radians": -0.09604986708678709,
              "degrees": -5.503252006865416,
              "degMinSec": {},
              "vrc": "E005.30.011.707",
              "nats": "0053012W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "center": {
            "lat": {
              "angle": {
                "radians": 0.9567671245480275,
                "degrees": 54.81871821346956,
                "percentage": 141.85740294405875,
                "degMinSec": {}
              },
              "radians": 0.9567671245480275,
              "degrees": 54.81871821346956,
              "degMinSec": {},
              "vrc": "N054.49.007.386",
              "nats": "544907N"
            },
            "lon": {
              "angle": {
                "radians": -0.09549379951414849,
                "degrees": -5.4713916818291395,
                "percentage": -9.578513365039377,
                "degMinSec": {}
              },
              "radians": -0.09549379951414849,
              "degrees": -5.4713916818291395,
              "degMinSec": {},
              "vrc": "E005.28.017.010",
              "nats": "0052817W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "radius_m": 6472.197705558189,
          "startTrueBearing": 341.6424505848206,
          "endTrueBearing": 161.67978256399155,
          "clockwise": true,
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9577312372084053,
                "degrees": 54.87395779988433,
                "percentage": 142.14822547879896,
                "degMinSec": {}
              },
              "radians": 0.9577312372084053,
              "degrees": 54.87395779988433,
              "degMinSec": {},
              "vrc": "N054.52.026.248",
              "nats": "545226N"
            },
            "lon": {
              "angle": {
                "radians": -0.09604986708678709,
                "degrees": -5.503252006865416,
                "percentage": -9.634633297626038,
                "degMinSec": {}
              },
              "radians": -0.09604986708678709,
              "degrees": -5.503252006865416,
              "degMinSec": {},
              "vrc": "E005.30.011.707",
              "nats": "0053012W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9558026586935596,
                "degrees": 54.76345839052407,
                "percentage": 141.56726871466853,
                "degMinSec": {}
              },
              "radians": 0.9558026586935596,
              "degrees": 54.76345839052407,
              "degMinSec": {},
              "vrc": "N054.45.048.450",
              "nats": "544548N"
            },
            "lon": {
              "angle": {
                "radians": -0.09494034016362995,
                "degrees": -5.439680796912376,
                "percentage": -9.522662597678853,
                "degMinSec": {}
              },
              "radians": -0.09494034016362995,
              "degrees": -5.439680796912376,
              "degMinSec": {},
              "vrc": "E005.26.022.851",
              "nats": "0052623W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9567510322293694,
                "degrees": 54.81779619152787,
                "percentage": 141.85255548144173,
                "degMinSec": {}
              },
              "radians": 0.9567510322293694,
              "degrees": 54.81779619152787,
              "degMinSec": {},
              "vrc": "N054.49.004.066",
              "nats": "544904N"
            },
            "lon": {
              "angle": {
                "radians": -0.0954866044417777,
                "degrees": -5.470979434549002,
                "percentage": -9.577787256973586,
                "degMinSec": {}
              },
              "radians": -0.0954866044417777,
              "degrees": -5.470979434549002,
              "degMinSec": {},
              "vrc": "E005.28.015.526",
              "nats": "0052816W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 7556.372918630817,
      "crossTrackDistance_m": -6035.207240762473,
      "requiredTrueCourse": 251.6848348413851,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 316,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.515240436302081,
          "degrees": 316,
          "percentage": -96.56887748070749,
          "degMinSec": {}
        },
        "radians": 5.515240436302081,
        "degrees": 316
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "ALT",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "NPT014V",
    "delayMs": -1,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "B738",
      "filedTas": 420,
      "origin": "EGAA",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 27000,
      "destination": "EGNX",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "LISBO L603 PEPOD Q38 MAKUX"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9286038717305959,
          "degrees": 53.2050826896708,
          "percentage": 133.69748386185879,
          "degMinSec": {}
        },
        "radians": 0.9286038717305959,
        "degrees": 53.2050826896708,
        "degMinSec": {},
        "vrc": "N053.12.018.298",
        "nats": "531218N"
      },
      "longitude": {
        "angle": {
          "radians": -0.05504451600557658,
          "degrees": -3.1538184524598467,
          "percentage": -5.510017656816917,
          "degMinSec": {}
        },
        "radians": -0.05504451600557658,
        "degrees": -3.1538184524598467,
        "degMinSec": {},
        "vrc": "E003.09.013.746",
        "nats": "0030914W"
      },
      "indicatedAltitude": {
        "meters": 7619.90664554169,
        "feet": 24999.694518958997,
        "nauticalMiles": 4.114420434957716,
        "statuteMiles": 4.734790477077424
      },
      "trueAltitude": {
        "meters": 7640.58280116669,
        "feet": 25067.529677379724,
        "nauticalMiles": 4.125584665856744,
        "statuteMiles": 4.7476380445490145
      },
      "pressureAltitude": {
        "meters": 7619.90664554169,
        "feet": 24999.694518958997,
        "nauticalMiles": 4.114420434957716,
        "statuteMiles": 4.734790477077424
      },
      "densityAltitude": {
        "meters": 7569.656277941504,
        "feet": 24834.831102921606,
        "nauticalMiles": 4.087287407095845,
        "statuteMiles": 4.703566346251332
      },
      "heading_Mag": {
        "angle": {
          "radians": 2.247047143255779,
          "degrees": 128.74631767548462,
          "percentage": -124.61382328107462,
          "degMinSec": {}
        },
        "radians": 2.247047143255779,
        "degrees": 128.74631767548462
      },
      "heading_True": {
        "angle": {
          "radians": 2.2429653069320565,
          "degrees": 128.51244568147212,
          "percentage": -125.66119244494094,
          "degMinSec": {}
        },
        "radians": 2.2429653069320565,
        "degrees": 128.51244568147212
      },
      "track_True": {
        "angle": {
          "radians": 2.228640991876521,
          "degrees": 127.69172288437423,
          "percentage": -129.42351821082988,
          "degMinSec": {}
        },
        "radians": 2.228640991876521,
        "degrees": 127.69172288437423
      },
      "track_Mag": {
        "angle": {
          "radians": 2.2327228282002434,
          "degrees": 127.92559487838673,
          "percentage": -128.33734083218962,
          "degMinSec": {}
        },
        "radians": 2.2327228282002434,
        "degrees": 127.92559487838673
      },
      "bank": {
        "radians": 0.0008754333496995775,
        "degrees": 0.050158636182786084,
        "percentage": 0.08754335733388854,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.020492673117317634,
        "degrees": 1.1741436805635006,
        "percentage": 2.0495542229582937,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 149.18767980485057,
        "knots": 289.99757626257997,
        "feetPerMinute": 29367.654444656757
      },
      "trueAirSpeed": {
        "metersPerSecond": 214.8820365229548,
        "knots": 417.6971574029265,
        "feetPerMinute": 42299.614842358256
      },
      "groundSpeed": {
        "metersPerSecond": 213.28005268062745,
        "knots": 414.5831507229216,
        "feetPerMinute": 41984.26368220259
      },
      "machNumber": 0.6820126351136524,
      "verticalSpeed": {
        "metersPerSecond": -0.06466618994088741,
        "knots": -0.12570098531945434,
        "feetPerMinute": -12.729565356339663
      },
      "flightPathAngle": {
        "radians": -0.00030319848080752185,
        "degrees": -0.017371993305049294,
        "percentage": -0.030319849009846547,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 191.372571138368,
        "knots": 371.9984241718898,
        "feetPerMinute": 37671.767177616195
      },
      "velocity_Y": {
        "metersPerSecond": -0.06466618994088741,
        "knots": -0.12570098531945434,
        "feetPerMinute": -12.729565356339663
      },
      "velocity_Z": {
        "metersPerSecond": -94.15370352429856,
        "knots": -183.0201116734866,
        "feetPerMinute": -18534.19420023958
      },
      "heading_Velocity": {
        "radiansPerSecond": 4.025256928480598E-05,
        "degreesPerSecond": 0.0023063023345773134
      },
      "bank_Velocity": {
        "radiansPerSecond": 0.00012375259830201208,
        "degreesPerSecond": 0.00709050158648313
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0.00048282988047397557,
        "degreesPerSecond": 0.027664114373964797
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.001143878101443896,
        "knotsPerSecond": 0.0022235205842231084
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 101551.1171875,
        "hectopascals": 1015.511171875,
        "inchesOfMercury": 29.99146993133491
      },
      "windDirection": {
        "angle": {
          "radians": 3.3339113488673062,
          "degrees": 191.01904956086406,
          "percentage": 19.47253712427659,
          "degMinSec": {}
        },
        "radians": 3.3339113488673062,
        "degrees": 191.01904956086406
      },
      "windSpeed": {
        "metersPerSecond": 3.470153635454933,
        "knots": 6.745437323357258,
        "feetPerMinute": 683.1011312007577
      },
      "windXComp": {
        "metersPerSecond": -3.0782485316463926,
        "knots": -5.98363493874965,
        "feetPerMinute": -605.954454754005
      },
      "windHComp": {
        "metersPerSecond": 1.6019838423273551,
        "knots": 3.1140066800049753,
        "feetPerMinute": 315.3511601556768
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9293878266869805,
            "degrees": 53.25,
            "percentage": 133.91624077078822,
            "degMinSec": {}
          },
          "radians": 0.9293878266869805,
          "degrees": 53.25,
          "degMinSec": {},
          "vrc": "N053.15.000.000",
          "nats": "531500N"
        },
        "lon": {
          "angle": {
            "radians": -0.0567226680575974,
            "degrees": -3.2499694824218586,
            "percentage": -5.678358077797164,
            "degMinSec": {}
          },
          "radians": -0.0567226680575974,
          "degrees": -3.2499694824218586,
          "degMinSec": {},
          "vrc": "E003.14.059.890",
          "nats": "0031500W"
        },
        "geoPotentialHeight": {
          "meters": 7485.578125,
          "feet": 24558.984135625,
          "nauticalMiles": 4.041888836393088,
          "statuteMiles": 4.651322604116957
        },
        "levelPressure": {
          "pascals": 40000,
          "hectopascals": 400,
          "inchesOfMercury": 11.813349084465447
        },
        "temp": {
          "kelvin": 248.0194091796875,
          "celsius": -25.130590820312477
        },
        "v": {
          "metersPerSecond": 3.406176805496216,
          "knots": 6.621076346302986,
          "feetPerMinute": 670.5072666326523
        },
        "u": {
          "metersPerSecond": 0.66326904296875,
          "knots": 1.2892915495605468,
          "feetPerMinute": 130.56477641601563
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 101551.1171875,
          "hectopascals": 1015.511171875,
          "inchesOfMercury": 29.99146993133491
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9286038717305959,
            "degrees": 53.2050826896708,
            "percentage": 133.69748386185879,
            "degMinSec": {}
          },
          "radians": 0.9286038717305959,
          "degrees": 53.2050826896708,
          "degMinSec": {},
          "vrc": "N053.12.018.298",
          "nats": "531218N"
        },
        "lon": {
          "angle": {
            "radians": -0.05504451600557658,
            "degrees": -3.1538184524598467,
            "percentage": -5.510017656816917,
            "degMinSec": {}
          },
          "radians": -0.05504451600557658,
          "degrees": -3.1538184524598467,
          "degMinSec": {},
          "vrc": "E003.09.013.746",
          "nats": "0030914W"
        },
        "alt": {
          "meters": 7640.58280116669,
          "feet": 25067.529677379724,
          "nauticalMiles": 4.125584665856744,
          "statuteMiles": 4.7476380445490145
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0.00012375259830201208,
        "degreesPerSecond": 0.00709050158648313
      },
      "pitchRate": {
        "radiansPerSecond": 0.00048282988047397557,
        "degreesPerSecond": 0.027664114373964797
      },
      "yawRate": {
        "radiansPerSecond": 4.025256928480598E-05,
        "degreesPerSecond": 0.0023063023345773134
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 61.62815853273543,
      "thrustLeverVel": 0.15257260061946454,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 27000,
      "departureAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGNX",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9220768363564562,
              "degrees": 52.83111111,
              "percentage": 131.8937859528108,
              "degMinSec": {}
            },
            "radians": 0.9220768363564562,
            "degrees": 52.83111111,
            "degMinSec": {},
            "vrc": "N052.49.052.000",
            "nats": "524952N"
          },
          "lon": {
            "angle": {
              "radians": -0.023174093995820044,
              "degrees": -1.3277777799999502,
              "percentage": -2.3178243348417236,
              "degMinSec": {}
            },
            "radians": -0.023174093995820044,
            "degrees": -1.3277777799999502,
            "degMinSec": {},
            "vrc": "E001.19.040.000",
            "nats": "0011940W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "legType": "TRACK_TO_FIX",
        "initialTrueCourse": {
          "angle": {
            "radians": 2.1975188376116463,
            "degrees": 125.90855479564185,
            "percentage": -138.10116580988242,
            "degMinSec": {}
          },
          "radians": 2.1975188376116463,
          "degrees": 125.90855479564185
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 2.204630786378992,
            "degrees": 126.31603944412402,
            "percentage": -136.05366033169258,
            "degMinSec": {}
          },
          "radians": 2.204630786378992,
          "degrees": 126.31603944412402
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "percentage": 137.51183446058306,
                  "degMinSec": {}
                },
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "degMinSec": {},
                "vrc": "N053.58.029.900",
                "nats": "535830N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "percentage": -8.528041502320681,
                  "degMinSec": {}
                },
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "degMinSec": {},
                "vrc": "E004.52.027.890",
                "nats": "0045228W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MAKUX"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9457992061636364,
                  "degrees": 54.19030278,
                  "percentage": 138.6039686407732,
                  "degMinSec": {}
                },
                "radians": 0.9457992061636364,
                "degrees": 54.19030278,
                "degMinSec": {},
                "vrc": "N054.11.025.090",
                "nats": "541125N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09385619561685488,
                  "degrees": -5.37756389000004,
                  "percentage": -9.41327627889322,
                  "degMinSec": {}
                },
                "radians": -0.09385619561685488,
                "degrees": -5.37756389000004,
                "degMinSec": {},
                "vrc": "E005.22.039.230",
                "nats": "0052239W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "VAKPO"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "legLength": {
          "meters": 40625.53453663855,
          "feet": 133285.87872918523,
          "nauticalMiles": 21.93603376708345,
          "statuteMiles": 25.243536830310084
        },
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9457992061636364,
                  "degrees": 54.19030278,
                  "percentage": 138.6039686407732,
                  "degMinSec": {}
                },
                "radians": 0.9457992061636364,
                "degrees": 54.19030278,
                "degMinSec": {},
                "vrc": "N054.11.025.090",
                "nats": "541125N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09385619561685488,
                  "degrees": -5.37756389000004,
                  "percentage": -9.41327627889322,
                  "degMinSec": {}
                },
                "radians": -0.09385619561685488,
                "degrees": -5.37756389000004,
                "degMinSec": {},
                "vrc": "E005.22.039.230",
                "nats": "0052239W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "percentage": 137.51183446058306,
                  "degMinSec": {}
                },
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "degMinSec": {},
                "vrc": "N053.58.029.900",
                "nats": "535830N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "percentage": -8.528041502320681,
                  "degMinSec": {}
                },
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "degMinSec": {},
                "vrc": "E004.52.027.890",
                "nats": "0045228W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [],
      "asString": "VAKPO =(TF)=> MAKUX; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9457992061636364,
                "degrees": 54.19030278,
                "percentage": 138.6039686407732,
                "degMinSec": {}
              },
              "radians": 0.9457992061636364,
              "degrees": 54.19030278,
              "degMinSec": {},
              "vrc": "N054.11.025.090",
              "nats": "541125N"
            },
            "lon": {
              "angle": {
                "radians": -0.09385619561685488,
                "degrees": -5.37756389000004,
                "percentage": -9.41327627889322,
                "degMinSec": {}
              },
              "radians": -0.09385619561685488,
              "degrees": -5.37756389000004,
              "degMinSec": {},
              "vrc": "E005.22.039.230",
              "nats": "0052239W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "percentage": 137.51183446058306,
                "degMinSec": {}
              },
              "radians": 0.9420409789114731,
              "degrees": 53.97497222,
              "degMinSec": {},
              "vrc": "N053.58.029.900",
              "nats": "535830N"
            },
            "lon": {
              "angle": {
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "percentage": -8.528041502320681,
                "degMinSec": {}
              },
              "radians": -0.0850745714854444,
              "degrees": -4.874413889999983,
              "degMinSec": {},
              "vrc": "E004.52.027.890",
              "nats": "0045228W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -142036.51105229062,
      "crossTrackDistance_m": 0.4157878704602074,
      "requiredTrueCourse": 127.69758909969457,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 130,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 2.2689280275926293,
          "degrees": 130.00000000000006,
          "percentage": -119.17535925942077,
          "degMinSec": {}
        },
        "radians": 2.2689280275926293,
        "degrees": 130.00000000000006
      },
      "selectedAltitude": 25000,
      "selectedAltitudeLength": {
        "meters": 7619.999756160008,
        "feet": 25000,
        "nauticalMiles": 4.114470710669551,
        "statuteMiles": 4.734848333333338
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "ALT",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "SHT5V",
    "delayMs": -1,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A319",
      "filedTas": 460,
      "origin": "EGAC",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 27000,
      "destination": "EGLL",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "LISBO L603 PEPOD Q38 MAKUX DCT SOSIM Q38 NUGRA"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9532560803632874,
          "degrees": 54.6175502,
          "percentage": 140.80499015231928,
          "degMinSec": {}
        },
        "radians": 0.9532560803632874,
        "degrees": 54.6175502,
        "degMinSec": {},
        "vrc": "N054.37.003.181",
        "nats": "543703N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10245083232805463,
          "degrees": -5.870000299999985,
          "percentage": -10.281079090017418,
          "degMinSec": {}
        },
        "radians": -0.10245083232805463,
        "degrees": -5.870000299999985,
        "degMinSec": {},
        "vrc": "E005.52.012.001",
        "nats": "0055212W"
      },
      "indicatedAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "trueAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "pressureAltitude": {
        "meters": 19.605781136035283,
        "feet": 64.32343098235,
        "nauticalMiles": 0.010586274911466135,
        "statuteMiles": 0.012182467599242476
      },
      "densityAltitude": {
        "meters": -116.76888099363045,
        "feet": -383.1000155191425,
        "nauticalMiles": -0.06305015172442249,
        "statuteMiles": -0.07255681879923151
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.835262917116978,
          "degrees": 334.33593749999994,
          "percentage": -48.04952269840091,
          "degMinSec": {}
        },
        "radians": 5.835262917116978,
        "degrees": 334.33593749999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.80959891749545,
          "degrees": 332.865498636261,
          "percentage": -51.24859344565099,
          "degMinSec": {}
        },
        "radians": 5.80959891749545,
        "degrees": 332.865498636261
      },
      "track_True": {
        "angle": {
          "radians": 5.80959891749545,
          "degrees": 332.865498636261,
          "percentage": -51.24859344565099,
          "degMinSec": {}
        },
        "radians": 5.80959891749545,
        "degrees": 332.865498636261
      },
      "track_Mag": {
        "angle": {
          "radians": 5.835262917116978,
          "degrees": 334.33593749999994,
          "percentage": -48.04952269840091,
          "degMinSec": {}
        },
        "radians": 5.835262917116978,
        "degrees": 334.33593749999994
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 10.32908164244109,
        "knots": 20.078123376169255,
        "feetPerMinute": 2033.2838529471856
      },
      "trueAirSpeed": {
        "metersPerSecond": -10.271429387583698,
        "knots": -19.966056386478243,
        "feetPerMinute": -2021.9349835176058
      },
      "groundSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "machNumber": -0.030300698455381293,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -0,
        "knots": -0,
        "feetPerMinute": -0
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 102003.921875,
        "hectopascals": 1020.03921875,
        "inchesOfMercury": 30.125198427347904
      },
      "surfacePressure": {
        "pascals": 102003.921875,
        "hectopascals": 1020.03921875,
        "inchesOfMercury": 30.125198427347904
      },
      "windDirection": {
        "angle": {
          "radians": 2.888472549290217,
          "degrees": 165.4972863137231,
          "percentage": -25.86681155013803,
          "degMinSec": {}
        },
        "radians": 2.888472549290217,
        "degrees": 165.4972863137231
      },
      "windSpeed": {
        "metersPerSecond": 10.526210135989794,
        "knots": 20.461310415582943,
        "feetPerMinute": 2072.0886757536455
      },
      "windXComp": {
        "metersPerSecond": 2.3019205379196492,
        "knots": 4.474574426111882,
        "feetPerMinute": 453.1339786576981
      },
      "windHComp": {
        "metersPerSecond": -10.271429387583698,
        "knots": -19.966056386478243,
        "feetPerMinute": -2021.9349835176058
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9512044423369096,
            "degrees": 54.5,
            "percentage": 140.19482944763357,
            "degMinSec": {}
          },
          "radians": 0.9512044423369096,
          "degrees": 54.5,
          "degMinSec": {},
          "vrc": "N054.30.000.000",
          "nats": "543000N"
        },
        "lon": {
          "angle": {
            "radians": -0.10035696462189136,
            "degrees": -5.75003051757809,
            "percentage": -10.069524321626655,
            "degMinSec": {}
          },
          "radians": -0.10035696462189136,
          "degrees": -5.75003051757809,
          "degMinSec": {},
          "vrc": "E005.45.000.110",
          "nats": "0054500W"
        },
        "geoPotentialHeight": {
          "meters": 220.80575561523438,
          "feet": 724.4283552526855,
          "nauticalMiles": 0.11922556998662763,
          "statuteMiles": 0.13720233561950357
        },
        "levelPressure": {
          "pascals": 100000,
          "hectopascals": 1000,
          "inchesOfMercury": 29.533372711163615
        },
        "temp": {
          "kelvin": 285.0251159667969,
          "celsius": 11.875115966796898
        },
        "v": {
          "metersPerSecond": 10.190800666809082,
          "knots": 19.80932673137283,
          "feetPerMinute": 2006.0631875816346
        },
        "u": {
          "metersPerSecond": -2.636035203933716,
          "knots": -5.12404121495533,
          "feetPerMinute": -518.9045843084335
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102003.921875,
          "hectopascals": 1020.03921875,
          "inchesOfMercury": 30.125198427347904
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9532560803632874,
            "degrees": 54.6175502,
            "percentage": 140.80499015231928,
            "degMinSec": {}
          },
          "radians": 0.9532560803632874,
          "degrees": 54.6175502,
          "degMinSec": {},
          "vrc": "N054.37.003.181",
          "nats": "543703N"
        },
        "lon": {
          "angle": {
            "radians": -0.10245083232805463,
            "degrees": -5.870000299999985,
            "percentage": -10.281079090017418,
            "degMinSec": {}
          },
          "radians": -0.10245083232805463,
          "degrees": -5.870000299999985,
          "degMinSec": {},
          "vrc": "E005.52.012.001",
          "nats": "0055212W"
        },
        "alt": {
          "meters": 81.68639738603528,
          "feet": 268,
          "nauticalMiles": 0.04410712601837758,
          "statuteMiles": 0.050757574133333386
        }
      },
      "onGround": true,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 27000,
      "departureAirport": {
        "identifier": "EGAC",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9532649005591953,
              "degrees": 54.61805556,
              "percentage": 140.80762090061415,
              "degMinSec": {}
            },
            "radians": 0.9532649005591953,
            "degrees": 54.61805556,
            "degMinSec": {},
            "vrc": "N054.37.005.000",
            "nats": "543705N"
          },
          "lon": {
            "angle": {
              "radians": -0.10249446032336706,
              "degrees": -5.872500000000004,
              "percentage": -10.285488024374672,
              "degMinSec": {}
            },
            "radians": -0.10249446032336706,
            "degrees": -5.872500000000004,
            "degMinSec": {},
            "vrc": "E005.52.021.000",
            "nats": "0055221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGLL",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.8984518656953809,
              "degrees": 51.4775,
              "percentage": 125.61594459166743,
              "degMinSec": {}
            },
            "radians": 0.8984518656953809,
            "degrees": 51.4775,
            "degMinSec": {},
            "vrc": "N051.28.039.000",
            "nats": "512839N"
          },
          "lon": {
            "angle": {
              "radians": -0.008052755262621503,
              "degrees": -0.46138888999997496,
              "percentage": -0.8052929332454014,
              "degMinSec": {}
            },
            "radians": -0.008052755262621503,
            "degrees": -0.46138888999997496,
            "degMinSec": {},
            "vrc": "E000.27.041.000",
            "nats": "0002741E"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9532559382614316,
                  "degrees": 54.6175420581634,
                  "percentage": 140.80494776896398,
                  "degMinSec": {}
                },
                "radians": 0.9532559382614316,
                "degrees": 54.6175420581634,
                "degMinSec": {},
                "vrc": "N054.37.003.151",
                "nats": "543703N"
              },
              "lon": {
                "angle": {
                  "radians": -0.1024513157449789,
                  "degrees": -5.8700279977494905,
                  "percentage": -10.2811279426868,
                  "degMinSec": {}
                },
                "radians": -0.1024513157449789,
                "degrees": -5.8700279977494905,
                "degMinSec": {},
                "vrc": "E005.52.012.101",
                "nats": "0055212W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPOD"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 2.6718099131429565,
            "degrees": 153.08343168430648,
            "percentage": -50.769260840582454,
            "degMinSec": {}
          },
          "radians": 2.6718099131429565,
          "degrees": 153.08343168430648
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 2.6760808853295313,
            "degrees": 153.32814036501497,
            "percentage": -50.23323757359711,
            "degMinSec": {}
          },
          "radians": 2.6760808853295313,
          "degrees": 153.32814036501497
        },
        "legLength": {
          "meters": 43143.982488125286,
          "feet": 141548.50350634096,
          "nauticalMiles": 23.295886872637844,
          "statuteMiles": 26.808427836513065
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9532559382614316,
                  "degrees": 54.6175420581634,
                  "percentage": 140.80494776896398,
                  "degMinSec": {}
                },
                "radians": 0.9532559382614316,
                "degrees": 54.6175420581634,
                "degMinSec": {},
                "vrc": "N054.37.003.151",
                "nats": "543703N"
              },
              "lon": {
                "angle": {
                  "radians": -0.1024513157449789,
                  "degrees": -5.8700279977494905,
                  "percentage": -10.2811279426868,
                  "degMinSec": {}
                },
                "radians": -0.1024513157449789,
                "degrees": -5.8700279977494905,
                "degMinSec": {},
                "vrc": "E005.52.012.101",
                "nats": "0055212W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.194801594813155,
              "degrees": 125.75286825137599,
              "percentage": -138.89409825386858,
              "degMinSec": {}
            },
            "radians": 2.194801594813155,
            "degrees": 125.75286825137599
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.204627895223652,
              "degrees": 126.31587379312512,
              "percentage": -136.05448462062563,
              "degMinSec": {}
            },
            "radians": 2.204627895223652,
            "degrees": 126.31587379312512
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAKUX"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPOD"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 55991.55979285222,
            "feet": 183699.34903078128,
            "nauticalMiles": 30.233023646248498,
            "statuteMiles": 34.79154226371256
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.205016557540862,
              "degrees": 126.33814250355704,
              "percentage": -135.94373235552416,
              "degMinSec": {}
            },
            "radians": 2.205016557540862,
            "degrees": 126.33814250355704
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.2101769520283945,
              "degrees": 126.63381132831522,
              "percentage": -134.4842416786908,
              "degMinSec": {}
            },
            "radians": 2.2101769520283945,
            "degrees": 126.63381132831522
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "SOSIM"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAKUX"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 29823.73049294273,
            "feet": 97846.88795046623,
            "nauticalMiles": 16.103526184094346,
            "statuteMiles": 18.531606973364756
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.094384425312917,
              "degrees": 119.99938824836254,
              "percentage": -173.20935166798225,
              "degMinSec": {}
            },
            "radians": 2.094384425312917,
            "degrees": 119.99938824836254
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.125287434425406,
              "degrees": 121.77000024476247,
              "percentage": -161.47220724121502,
              "degMinSec": {}
            },
            "radians": 2.125287434425406,
            "degrees": 121.77000024476247
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.925540490777192,
                    "degrees": 53.02956389,
                    "percentage": 132.84704633992573,
                    "degMinSec": {}
                  },
                  "radians": 0.925540490777192,
                  "degrees": 53.02956389,
                  "degMinSec": {},
                  "vrc": "N053.01.046.430",
                  "nats": "530146N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.04020811968312543,
                    "degrees": -2.3037555599999804,
                    "percentage": -4.022980176492362,
                    "degMinSec": {}
                  },
                  "radians": -0.04020811968312543,
                  "degrees": -2.3037555599999804,
                  "degMinSec": {},
                  "vrc": "E002.18.013.520",
                  "nats": "0021814W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "NUGRA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "SOSIM"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 170220.64776857995,
            "feet": 558466.7100250678,
            "nauticalMiles": 91.91179685128508,
            "statuteMiles": 105.7702068473738
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.925540490777192,
                    "degrees": 53.02956389,
                    "percentage": 132.84704633992573,
                    "degMinSec": {}
                  },
                  "radians": 0.925540490777192,
                  "degrees": 53.02956389,
                  "degMinSec": {},
                  "vrc": "N053.01.046.430",
                  "nats": "530146N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.04020811968312543,
                    "degrees": -2.3037555599999804,
                    "percentage": -4.022980176492362,
                    "degMinSec": {}
                  },
                  "radians": -0.04020811968312543,
                  "degrees": -2.3037555599999804,
                  "degMinSec": {},
                  "vrc": "E002.18.013.520",
                  "nats": "0021814W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPOD; PEPOD =(TF)=> MAKUX; MAKUX =(TF)=> SOSIM; SOSIM =(TF)=> NUGRA; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9532559382614316,
                "degrees": 54.6175420581634,
                "percentage": 140.80494776896398,
                "degMinSec": {}
              },
              "radians": 0.9532559382614316,
              "degrees": 54.6175420581634,
              "degMinSec": {},
              "vrc": "N054.37.003.151",
              "nats": "543703N"
            },
            "lon": {
              "angle": {
                "radians": -0.1024513157449789,
                "degrees": -5.8700279977494905,
                "percentage": -10.2811279426868,
                "degMinSec": {}
              },
              "radians": -0.1024513157449789,
              "degrees": -5.8700279977494905,
              "degMinSec": {},
              "vrc": "E005.52.012.101",
              "nats": "0055212W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "percentage": 137.51183446058306,
                "degMinSec": {}
              },
              "radians": 0.9420409789114731,
              "degrees": 53.97497222,
              "degMinSec": {},
              "vrc": "N053.58.029.900",
              "nats": "535830N"
            },
            "lon": {
              "angle": {
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "percentage": -8.528041502320681,
                "degMinSec": {}
              },
              "radians": -0.0850745714854444,
              "degrees": -4.874413889999983,
              "degMinSec": {},
              "vrc": "E004.52.027.890",
              "nats": "0045228W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "percentage": 137.51183446058306,
                "degMinSec": {}
              },
              "radians": 0.9420409789114731,
              "degrees": 53.97497222,
              "degMinSec": {},
              "vrc": "N053.58.029.900",
              "nats": "535830N"
            },
            "lon": {
              "angle": {
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "percentage": -8.528041502320681,
                "degMinSec": {}
              },
              "radians": -0.0850745714854444,
              "degrees": -4.874413889999983,
              "degMinSec": {},
              "vrc": "E004.52.027.890",
              "nats": "0045228W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9392574211807754,
                "degrees": 53.81548611,
                "percentage": 136.71018811825857,
                "degMinSec": {}
              },
              "radians": 0.9392574211807754,
              "degrees": 53.81548611,
              "degMinSec": {},
              "vrc": "N053.48.055.750",
              "nats": "534856N"
            },
            "lon": {
              "angle": {
                "radians": -0.07868749056761892,
                "degrees": -4.508461110000039,
                "percentage": -7.8850297468587085,
                "degMinSec": {}
              },
              "radians": -0.07868749056761892,
              "degrees": -4.508461110000039,
              "degMinSec": {},
              "vrc": "E004.30.030.460",
              "nats": "0043030W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9392574211807754,
                "degrees": 53.81548611,
                "percentage": 136.71018811825857,
                "degMinSec": {}
              },
              "radians": 0.9392574211807754,
              "degrees": 53.81548611,
              "degMinSec": {},
              "vrc": "N053.48.055.750",
              "nats": "534856N"
            },
            "lon": {
              "angle": {
                "radians": -0.07868749056761892,
                "degrees": -4.508461110000039,
                "percentage": -7.8850297468587085,
                "degMinSec": {}
              },
              "radians": -0.07868749056761892,
              "degrees": -4.508461110000039,
              "degMinSec": {},
              "vrc": "E004.30.030.460",
              "nats": "0043030W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.925540490777192,
                "degrees": 53.02956389,
                "percentage": 132.84704633992573,
                "degMinSec": {}
              },
              "radians": 0.925540490777192,
              "degrees": 53.02956389,
              "degMinSec": {},
              "vrc": "N053.01.046.430",
              "nats": "530146N"
            },
            "lon": {
              "angle": {
                "radians": -0.04020811968312543,
                "degrees": -2.3037555599999804,
                "percentage": -4.022980176492362,
                "degMinSec": {}
              },
              "radians": -0.04020811968312543,
              "degrees": -2.3037555599999804,
              "degMinSec": {},
              "vrc": "E002.18.013.520",
              "nats": "0021814W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 334,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.82939970166106,
          "degrees": 334,
          "percentage": -48.773258856586196,
          "degMinSec": {}
        },
        "radians": 5.82939970166106,
        "degrees": 334
      },
      "selectedAltitude": 3000,
      "selectedAltitudeLength": {
        "meters": 914.399970739201,
        "feet": 3000,
        "nauticalMiles": 0.4937364852803461,
        "statuteMiles": 0.5681818000000006
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "ON_GROUND"
  },
  {
    "callsign": "EZY904D",
    "delayMs": 240000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A319",
      "filedTas": 420,
      "origin": "EGAC",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 37000,
      "destination": "EGGW",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "DUFFY L15 PEPOD Q38 MAKUX DCT KELLY L10 HON L15 FINMA"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9532505616321927,
          "degrees": 54.617234,
          "percentage": 140.8033441458762,
          "degMinSec": {}
        },
        "radians": 0.9532505616321927,
        "degrees": 54.617234,
        "degMinSec": {},
        "vrc": "N054.37.002.042",
        "nats": "543702N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10245000155133077,
          "degrees": -5.86995269999999,
          "percentage": -10.280995134216326,
          "degMinSec": {}
        },
        "radians": -0.10245000155133077,
        "degrees": -5.86995269999999,
        "degMinSec": {},
        "vrc": "E005.52.011.830",
        "nats": "0055212W"
      },
      "indicatedAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "trueAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "pressureAltitude": {
        "meters": 19.605781136035283,
        "feet": 64.32343098235,
        "nauticalMiles": 0.010586274911466135,
        "statuteMiles": 0.012182467599242476
      },
      "densityAltitude": {
        "meters": -116.76888099363045,
        "feet": -383.1000155191425,
        "nauticalMiles": -0.06305015172442249,
        "statuteMiles": -0.07255681879923151
      },
      "heading_Mag": {
        "angle": {
          "radians": 0.6013204688511715,
          "degrees": 34.45312500000001,
          "percentage": 68.60770675448632,
          "degMinSec": {}
        },
        "radians": 0.6013204688511715,
        "degrees": 34.45312500000001
      },
      "heading_True": {
        "angle": {
          "radians": 0.575656469229644,
          "degrees": 32.98268613626114,
          "percentage": 64.89780534537857,
          "degMinSec": {}
        },
        "radians": 0.575656469229644,
        "degrees": 32.98268613626114
      },
      "track_True": {
        "angle": {
          "radians": 0.575656469229644,
          "degrees": 32.98268613626114,
          "percentage": 64.89780534537857,
          "degMinSec": {}
        },
        "radians": 0.575656469229644,
        "degrees": 32.98268613626114
      },
      "track_Mag": {
        "angle": {
          "radians": 0.6013204688511724,
          "degrees": 34.453125000000064,
          "percentage": 68.60770675448644,
          "degMinSec": {}
        },
        "radians": 0.6013204688511724,
        "degrees": 34.453125000000064
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 7.153309866880296,
        "knots": 13.904918464876062,
        "feetPerMinute": 1408.131908619333
      },
      "trueAirSpeed": {
        "metersPerSecond": -7.113381860130174,
        "knots": -13.827304648522876,
        "feetPerMinute": -1400.2720645193688
      },
      "groundSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "machNumber": -0.02098446385683507,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 102003.921875,
        "hectopascals": 1020.03921875,
        "inchesOfMercury": 30.125198427347904
      },
      "surfacePressure": {
        "pascals": 102003.921875,
        "hectopascals": 1020.03921875,
        "inchesOfMercury": 30.125198427347904
      },
      "windDirection": {
        "angle": {
          "radians": 2.888472549290217,
          "degrees": 165.4972863137231,
          "percentage": -25.86681155013803,
          "degMinSec": {}
        },
        "radians": 2.888472549290217,
        "degrees": 165.4972863137231
      },
      "windSpeed": {
        "metersPerSecond": 10.526210135989794,
        "knots": 20.461310415582943,
        "feetPerMinute": 2072.0886757536455
      },
      "windXComp": {
        "metersPerSecond": -7.758923787419573,
        "knots": -15.082137450632812,
        "feetPerMinute": -1527.347251123058
      },
      "windHComp": {
        "metersPerSecond": -7.113381860130174,
        "knots": -13.827304648522876,
        "feetPerMinute": -1400.2720645193688
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9512044423369096,
            "degrees": 54.5,
            "percentage": 140.19482944763357,
            "degMinSec": {}
          },
          "radians": 0.9512044423369096,
          "degrees": 54.5,
          "degMinSec": {},
          "vrc": "N054.30.000.000",
          "nats": "543000N"
        },
        "lon": {
          "angle": {
            "radians": -0.10035696462189136,
            "degrees": -5.75003051757809,
            "percentage": -10.069524321626655,
            "degMinSec": {}
          },
          "radians": -0.10035696462189136,
          "degrees": -5.75003051757809,
          "degMinSec": {},
          "vrc": "E005.45.000.110",
          "nats": "0054500W"
        },
        "geoPotentialHeight": {
          "meters": 220.80575561523438,
          "feet": 724.4283552526855,
          "nauticalMiles": 0.11922556998662763,
          "statuteMiles": 0.13720233561950357
        },
        "levelPressure": {
          "pascals": 100000,
          "hectopascals": 1000,
          "inchesOfMercury": 29.533372711163615
        },
        "temp": {
          "kelvin": 285.0251159667969,
          "celsius": 11.875115966796898
        },
        "v": {
          "metersPerSecond": 10.190800666809082,
          "knots": 19.80932673137283,
          "feetPerMinute": 2006.0631875816346
        },
        "u": {
          "metersPerSecond": -2.636035203933716,
          "knots": -5.12404121495533,
          "feetPerMinute": -518.9045843084335
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102003.921875,
          "hectopascals": 1020.03921875,
          "inchesOfMercury": 30.125198427347904
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9532505616321927,
            "degrees": 54.617234,
            "percentage": 140.8033441458762,
            "degMinSec": {}
          },
          "radians": 0.9532505616321927,
          "degrees": 54.617234,
          "degMinSec": {},
          "vrc": "N054.37.002.042",
          "nats": "543702N"
        },
        "lon": {
          "angle": {
            "radians": -0.10245000155133077,
            "degrees": -5.86995269999999,
            "percentage": -10.280995134216326,
            "degMinSec": {}
          },
          "radians": -0.10245000155133077,
          "degrees": -5.86995269999999,
          "degMinSec": {},
          "vrc": "E005.52.011.830",
          "nats": "0055212W"
        },
        "alt": {
          "meters": 81.68639738603528,
          "feet": 268,
          "nauticalMiles": 0.04410712601837758,
          "statuteMiles": 0.050757574133333386
        }
      },
      "onGround": true,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 37000,
      "departureAirport": {
        "identifier": "EGAC",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9532649005591953,
              "degrees": 54.61805556,
              "percentage": 140.80762090061415,
              "degMinSec": {}
            },
            "radians": 0.9532649005591953,
            "degrees": 54.61805556,
            "degMinSec": {},
            "vrc": "N054.37.005.000",
            "nats": "543705N"
          },
          "lon": {
            "angle": {
              "radians": -0.10249446032336706,
              "degrees": -5.872500000000004,
              "percentage": -10.285488024374672,
              "degMinSec": {}
            },
            "radians": -0.10249446032336706,
            "degrees": -5.872500000000004,
            "degMinSec": {},
            "vrc": "E005.52.021.000",
            "nats": "0055221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGGW",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9053847012964623,
              "degrees": 51.87472222,
              "percentage": 127.41891613234773,
              "degMinSec": {}
            },
            "radians": 0.9053847012964623,
            "degrees": 51.87472222,
            "degMinSec": {},
            "vrc": "N051.52.029.000",
            "nats": "515229N"
          },
          "lon": {
            "angle": {
              "radians": -0.0064286293533353245,
              "degrees": -0.36833333000002977,
              "percentage": -0.6428717914044477,
              "degMinSec": {}
            },
            "radians": -0.0064286293533353245,
            "degrees": -0.36833333000002977,
            "degMinSec": {},
            "vrc": "E000.22.006.000",
            "nats": "0002206E"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9532506683589632,
                  "degrees": 54.617240114993514,
                  "percentage": 140.80337597776116,
                  "degMinSec": {}
                },
                "radians": 0.9532506683589632,
                "degrees": 54.617240114993514,
                "degMinSec": {},
                "vrc": "N054.37.002.064",
                "nats": "543702N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10244963871730839,
                  "degrees": -5.869931911141843,
                  "percentage": -10.280958467304027,
                  "degMinSec": {}
                },
                "radians": -0.10244963871730839,
                "degrees": -5.869931911141843,
                "degMinSec": {},
                "vrc": "E005.52.011.755",
                "nats": "0055212W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPOD"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 2.6715866894260163,
            "degrees": 153.07064190743856,
            "percentage": -50.79734002607105,
            "degMinSec": {}
          },
          "radians": 2.6715866894260163,
          "degrees": 153.07064190743856
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 2.675856289180061,
            "degrees": 153.31527195355542,
            "percentage": -50.26136777289766,
            "degMinSec": {}
          },
          "radians": 2.675856289180061,
          "degrees": 153.31527195355542
        },
        "legLength": {
          "meters": 43111.245637475375,
          "feet": 141441.0991372547,
          "nauticalMiles": 23.27821038740571,
          "statuteMiles": 26.788086100594636
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9532506683589632,
                  "degrees": 54.617240114993514,
                  "percentage": 140.80337597776116,
                  "degMinSec": {}
                },
                "radians": 0.9532506683589632,
                "degrees": 54.617240114993514,
                "degMinSec": {},
                "vrc": "N054.37.002.064",
                "nats": "543702N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10244963871730839,
                  "degrees": -5.869931911141843,
                  "percentage": -10.280958467304027,
                  "degMinSec": {}
                },
                "radians": -0.10244963871730839,
                "degrees": -5.869931911141843,
                "degMinSec": {},
                "vrc": "E005.52.011.755",
                "nats": "0055212W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.194801594813155,
              "degrees": 125.75286825137599,
              "percentage": -138.89409825386858,
              "degMinSec": {}
            },
            "radians": 2.194801594813155,
            "degrees": 125.75286825137599
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.204627895223652,
              "degrees": 126.31587379312512,
              "percentage": -136.05448462062563,
              "degMinSec": {}
            },
            "radians": 2.204627895223652,
            "degrees": 126.31587379312512
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAKUX"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPOD"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 55991.55979285222,
            "feet": 183699.34903078128,
            "nauticalMiles": 30.233023646248498,
            "statuteMiles": 34.79154226371256
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 1.7947358110628038,
              "degrees": 102.83078731488737,
              "percentage": -439.0594854183536,
              "degMinSec": {}
            },
            "radians": 1.7947358110628038,
            "degrees": 102.83078731488737
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 1.8019369380856993,
              "degrees": 103.24338150103691,
              "percentage": -424.90481427335806,
              "degMinSec": {}
            },
            "radians": 1.8019369380856993,
            "degrees": 103.24338150103691
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.940826859990478,
                    "degrees": 53.90540833,
                    "percentage": 137.16142355393953,
                    "degMinSec": {}
                  },
                  "radians": 0.940826859990478,
                  "degrees": 53.90540833,
                  "degMinSec": {},
                  "vrc": "N053.54.019.470",
                  "nats": "535419N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07616675027527187,
                    "degrees": -4.364033329999979,
                    "percentage": -7.631438347258509,
                    "degMinSec": {}
                  },
                  "radians": -0.07616675027527187,
                  "degrees": -4.364033329999979,
                  "degMinSec": {},
                  "vrc": "E004.21.050.520",
                  "nats": "0042151W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "KELLY"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAKUX"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 34289.50593494791,
            "feet": 112498.3826516145,
            "nauticalMiles": 18.514852016710535,
            "statuteMiles": 21.306511184027723
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.940826859990478,
                    "degrees": 53.90540833,
                    "percentage": 137.16142355393953,
                    "degMinSec": {}
                  },
                  "radians": 0.940826859990478,
                  "degrees": 53.90540833,
                  "degMinSec": {},
                  "vrc": "N053.54.019.470",
                  "nats": "535419N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07616675027527187,
                    "degrees": -4.364033329999979,
                    "percentage": -7.631438347258509,
                    "degMinSec": {}
                  },
                  "radians": -0.07616675027527187,
                  "degrees": -4.364033329999979,
                  "degMinSec": {},
                  "vrc": "E004.21.050.520",
                  "nats": "0042151W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.314698105070269,
              "degrees": 132.6224322674556,
              "percentage": -108.6637653978104,
              "degMinSec": {}
            },
            "radians": 2.314698105070269,
            "degrees": 132.6224322674556
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.3524079417844703,
              "degrees": 134.7830467573068,
              "percentage": -100.76019181789324,
              "degMinSec": {}
            },
            "radians": 2.3524079417844703,
            "degrees": 134.7830467573068
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9137964126667553,
                    "degrees": 52.35667778,
                    "percentage": 129.64974025129806,
                    "degMinSec": {}
                  },
                  "radians": 0.9137964126667553,
                  "degrees": 52.35667778,
                  "degMinSec": {},
                  "vrc": "N052.21.024.040",
                  "nats": "522124N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.029037479097743457,
                    "degrees": -1.6637250000000456,
                    "percentage": -2.904564307856217,
                    "degMinSec": {}
                  },
                  "radians": -0.029037479097743457,
                  "degrees": -1.6637250000000456,
                  "degMinSec": {},
                  "vrc": "E001.39.049.410",
                  "nats": "0013949W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "HON"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.940826859990478,
                    "degrees": 53.90540833,
                    "percentage": 137.16142355393953,
                    "degMinSec": {}
                  },
                  "radians": 0.940826859990478,
                  "degrees": 53.90540833,
                  "degMinSec": {},
                  "vrc": "N053.54.019.470",
                  "nats": "535419N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07616675027527187,
                    "degrees": -4.364033329999979,
                    "percentage": -7.631438347258509,
                    "degMinSec": {}
                  },
                  "radians": -0.07616675027527187,
                  "degrees": -4.364033329999979,
                  "degMinSec": {},
                  "vrc": "E004.21.050.520",
                  "nats": "0042151W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "KELLY"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 249188.9865786488,
            "feet": 817549.1947266941,
            "nauticalMiles": 134.5512886493784,
            "statuteMiles": 154.83885768278802
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.940826859990478,
                    "degrees": 53.90540833,
                    "percentage": 137.16142355393953,
                    "degMinSec": {}
                  },
                  "radians": 0.940826859990478,
                  "degrees": 53.90540833,
                  "degMinSec": {},
                  "vrc": "N053.54.019.470",
                  "nats": "535419N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07616675027527187,
                    "degrees": -4.364033329999979,
                    "percentage": -7.631438347258509,
                    "degMinSec": {}
                  },
                  "radians": -0.07616675027527187,
                  "degrees": -4.364033329999979,
                  "degMinSec": {},
                  "vrc": "E004.21.050.520",
                  "nats": "0042151W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9137964126667553,
                    "degrees": 52.35667778,
                    "percentage": 129.64974025129806,
                    "degMinSec": {}
                  },
                  "radians": 0.9137964126667553,
                  "degrees": 52.35667778,
                  "degMinSec": {},
                  "vrc": "N052.21.024.040",
                  "nats": "522124N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.029037479097743457,
                    "degrees": -1.6637250000000456,
                    "percentage": -2.904564307856217,
                    "degMinSec": {}
                  },
                  "radians": -0.029037479097743457,
                  "degrees": -1.6637250000000456,
                  "degMinSec": {},
                  "vrc": "E001.39.049.410",
                  "nats": "0013949W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.333122536093134,
              "degrees": 133.67807440499564,
              "percentage": -104.72422623947719,
              "degMinSec": {}
            },
            "radians": 2.333122536093134,
            "degrees": 133.67807440499564
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.341532597575336,
              "degrees": 134.15993543337137,
              "percentage": -102.97622894336126,
              "degMinSec": {}
            },
            "radians": 2.341532597575336,
            "degrees": 134.15993543337137
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9075092033284524,
                    "degrees": 51.99644722000001,
                    "percentage": 127.97780533022234,
                    "degMinSec": {}
                  },
                  "radians": 0.9075092033284524,
                  "degrees": 51.99644722000001,
                  "degMinSec": {},
                  "vrc": "N051.59.047.210",
                  "nats": "515947N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.01839063136857,
                    "degrees": -1.053705559999962,
                    "percentage": -1.8392704980205534,
                    "degMinSec": {}
                  },
                  "radians": -0.01839063136857,
                  "degrees": -1.053705559999962,
                  "degMinSec": {},
                  "vrc": "E001.03.013.340",
                  "nats": "0010313W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "FINMA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9137964126667553,
                    "degrees": 52.35667778,
                    "percentage": 129.64974025129806,
                    "degMinSec": {}
                  },
                  "radians": 0.9137964126667553,
                  "degrees": 52.35667778,
                  "degMinSec": {},
                  "vrc": "N052.21.024.040",
                  "nats": "522124N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.029037479097743457,
                    "degrees": -1.6637250000000456,
                    "percentage": -2.904564307856217,
                    "degMinSec": {}
                  },
                  "radians": -0.029037479097743457,
                  "degrees": -1.6637250000000456,
                  "degMinSec": {},
                  "vrc": "E001.39.049.410",
                  "nats": "0013949W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "HON"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 57746.47989777996,
            "feet": 189456.9611078324,
            "nauticalMiles": 31.180604696425462,
            "statuteMiles": 35.88199906159277
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9137964126667553,
                    "degrees": 52.35667778,
                    "percentage": 129.64974025129806,
                    "degMinSec": {}
                  },
                  "radians": 0.9137964126667553,
                  "degrees": 52.35667778,
                  "degMinSec": {},
                  "vrc": "N052.21.024.040",
                  "nats": "522124N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.029037479097743457,
                    "degrees": -1.6637250000000456,
                    "percentage": -2.904564307856217,
                    "degMinSec": {}
                  },
                  "radians": -0.029037479097743457,
                  "degrees": -1.6637250000000456,
                  "degMinSec": {},
                  "vrc": "E001.39.049.410",
                  "nats": "0013949W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9075092033284524,
                    "degrees": 51.99644722000001,
                    "percentage": 127.97780533022234,
                    "degMinSec": {}
                  },
                  "radians": 0.9075092033284524,
                  "degrees": 51.99644722000001,
                  "degMinSec": {},
                  "vrc": "N051.59.047.210",
                  "nats": "515947N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.01839063136857,
                    "degrees": -1.053705559999962,
                    "percentage": -1.8392704980205534,
                    "degMinSec": {}
                  },
                  "radians": -0.01839063136857,
                  "degrees": -1.053705559999962,
                  "degMinSec": {},
                  "vrc": "E001.03.013.340",
                  "nats": "0010313W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPOD; PEPOD =(TF)=> MAKUX; MAKUX =(TF)=> KELLY; KELLY =(TF)=> HON; HON =(TF)=> FINMA; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9532506683589632,
                "degrees": 54.617240114993514,
                "percentage": 140.80337597776116,
                "degMinSec": {}
              },
              "radians": 0.9532506683589632,
              "degrees": 54.617240114993514,
              "degMinSec": {},
              "vrc": "N054.37.002.064",
              "nats": "543702N"
            },
            "lon": {
              "angle": {
                "radians": -0.10244963871730839,
                "degrees": -5.869931911141843,
                "percentage": -10.280958467304027,
                "degMinSec": {}
              },
              "radians": -0.10244963871730839,
              "degrees": -5.869931911141843,
              "degMinSec": {},
              "vrc": "E005.52.011.755",
              "nats": "0055212W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "percentage": 137.51183446058306,
                "degMinSec": {}
              },
              "radians": 0.9420409789114731,
              "degrees": 53.97497222,
              "degMinSec": {},
              "vrc": "N053.58.029.900",
              "nats": "535830N"
            },
            "lon": {
              "angle": {
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "percentage": -8.528041502320681,
                "degMinSec": {}
              },
              "radians": -0.0850745714854444,
              "degrees": -4.874413889999983,
              "degMinSec": {},
              "vrc": "E004.52.027.890",
              "nats": "0045228W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "percentage": 137.51183446058306,
                "degMinSec": {}
              },
              "radians": 0.9420409789114731,
              "degrees": 53.97497222,
              "degMinSec": {},
              "vrc": "N053.58.029.900",
              "nats": "535830N"
            },
            "lon": {
              "angle": {
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "percentage": -8.528041502320681,
                "degMinSec": {}
              },
              "radians": -0.0850745714854444,
              "degrees": -4.874413889999983,
              "degMinSec": {},
              "vrc": "E004.52.027.890",
              "nats": "0045228W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.940826859990478,
                "degrees": 53.90540833,
                "percentage": 137.16142355393953,
                "degMinSec": {}
              },
              "radians": 0.940826859990478,
              "degrees": 53.90540833,
              "degMinSec": {},
              "vrc": "N053.54.019.470",
              "nats": "535419N"
            },
            "lon": {
              "angle": {
                "radians": -0.07616675027527187,
                "degrees": -4.364033329999979,
                "percentage": -7.631438347258509,
                "degMinSec": {}
              },
              "radians": -0.07616675027527187,
              "degrees": -4.364033329999979,
              "degMinSec": {},
              "vrc": "E004.21.050.520",
              "nats": "0042151W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.940826859990478,
                "degrees": 53.90540833,
                "percentage": 137.16142355393953,
                "degMinSec": {}
              },
              "radians": 0.940826859990478,
              "degrees": 53.90540833,
              "degMinSec": {},
              "vrc": "N053.54.019.470",
              "nats": "535419N"
            },
            "lon": {
              "angle": {
                "radians": -0.07616675027527187,
                "degrees": -4.364033329999979,
                "percentage": -7.631438347258509,
                "degMinSec": {}
              },
              "radians": -0.07616675027527187,
              "degrees": -4.364033329999979,
              "degMinSec": {},
              "vrc": "E004.21.050.520",
              "nats": "0042151W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9137964126667553,
                "degrees": 52.35667778,
                "percentage": 129.64974025129806,
                "degMinSec": {}
              },
              "radians": 0.9137964126667553,
              "degrees": 52.35667778,
              "degMinSec": {},
              "vrc": "N052.21.024.040",
              "nats": "522124N"
            },
            "lon": {
              "angle": {
                "radians": -0.029037479097743457,
                "degrees": -1.6637250000000456,
                "percentage": -2.904564307856217,
                "degMinSec": {}
              },
              "radians": -0.029037479097743457,
              "degrees": -1.6637250000000456,
              "degMinSec": {},
              "vrc": "E001.39.049.410",
              "nats": "0013949W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9137964126667553,
                "degrees": 52.35667778,
                "percentage": 129.64974025129806,
                "degMinSec": {}
              },
              "radians": 0.9137964126667553,
              "degrees": 52.35667778,
              "degMinSec": {},
              "vrc": "N052.21.024.040",
              "nats": "522124N"
            },
            "lon": {
              "angle": {
                "radians": -0.029037479097743457,
                "degrees": -1.6637250000000456,
                "percentage": -2.904564307856217,
                "degMinSec": {}
              },
              "radians": -0.029037479097743457,
              "degrees": -1.6637250000000456,
              "degMinSec": {},
              "vrc": "E001.39.049.410",
              "nats": "0013949W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9075092033284524,
                "degrees": 51.99644722000001,
                "percentage": 127.97780533022234,
                "degMinSec": {}
              },
              "radians": 0.9075092033284524,
              "degrees": 51.99644722000001,
              "degMinSec": {},
              "vrc": "N051.59.047.210",
              "nats": "515947N"
            },
            "lon": {
              "angle": {
                "radians": -0.01839063136857,
                "degrees": -1.053705559999962,
                "percentage": -1.8392704980205534,
                "degMinSec": {}
              },
              "radians": -0.01839063136857,
              "degrees": -1.053705559999962,
              "degMinSec": {},
              "vrc": "E001.03.013.340",
              "nats": "0010313W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 34,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 0.5934119456780724,
          "degrees": 34.00000000000002,
          "percentage": 67.45085168424272,
          "degMinSec": {}
        },
        "radians": 0.5934119456780724,
        "degrees": 34.00000000000002
      },
      "selectedAltitude": 3000,
      "selectedAltitudeLength": {
        "meters": 914.399970739201,
        "feet": 3000,
        "nauticalMiles": 0.4937364852803461,
        "statuteMiles": 0.5681818000000006
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "ON_GROUND"
  },
  {
    "callsign": "RYR79TT",
    "delayMs": 300000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "B738",
      "filedTas": 450,
      "origin": "EPKK",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 30000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "MIMVA L602 NALAX L46 GETNO DCT REMSI DCT UVPOK M147 ROBOP M146 IPSET P6 BELZU DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9470457077023122,
          "degrees": 54.26172205732269,
          "percentage": 138.9687153166313,
          "degMinSec": {}
        },
        "radians": 0.9470457077023122,
        "degrees": 54.26172205732269,
        "degMinSec": {},
        "vrc": "N054.15.042.199",
        "nats": "541542N"
      },
      "longitude": {
        "angle": {
          "radians": -0.09520870863877029,
          "degrees": -5.455057177892279,
          "percentage": -9.549743497410965,
          "degMinSec": {}
        },
        "radians": -0.09520870863877029,
        "degrees": -5.455057177892279,
        "degMinSec": {},
        "vrc": "E005.27.018.206",
        "nats": "0052718W"
      },
      "indicatedAltitude": {
        "meters": 3047.894795985524,
        "feet": 9999.655162461147,
        "nauticalMiles": 1.6457315313096783,
        "statuteMiles": 1.893874023195491
      },
      "trueAltitude": {
        "meters": 3166.8875253605243,
        "feet": 10390.051268703823,
        "nauticalMiles": 1.7099824650974753,
        "statuteMiles": 1.967812677314809
      },
      "pressureAltitude": {
        "meters": 3047.894795985524,
        "feet": 9999.655162461147,
        "nauticalMiles": 1.6457315313096783,
        "statuteMiles": 1.893874023195491
      },
      "densityAltitude": {
        "meters": 3336.3376342012334,
        "feet": 10945.989963792774,
        "nauticalMiles": 1.801478204212329,
        "statuteMiles": 2.0731040934699068
      },
      "heading_Mag": {
        "angle": {
          "radians": 3.1605596396570377,
          "degrees": 181.08672825173656,
          "percentage": 1.8969260830580406,
          "degMinSec": {}
        },
        "radians": 3.1605596396570377,
        "degrees": 181.08672825173656
      },
      "heading_True": {
        "angle": {
          "radians": 3.138754164059119,
          "degrees": 179.8373665297003,
          "percentage": -0.28384971539573317,
          "degMinSec": {}
        },
        "radians": 3.138754164059119,
        "degrees": 179.8373665297003
      },
      "track_True": {
        "angle": {
          "radians": 3.1473882303440064,
          "degrees": 180.33206208786055,
          "percentage": 0.5795641643733765,
          "degMinSec": {}
        },
        "radians": 3.1473882303440064,
        "degrees": 180.33206208786055
      },
      "track_Mag": {
        "angle": {
          "radians": 3.1691937059419253,
          "degrees": 181.58142380989682,
          "percentage": 2.760806348229244,
          "degMinSec": {}
        },
        "radians": 3.1691937059419253,
        "degrees": 181.58142380989682
      },
      "bank": {
        "radians": 6.55281028442028E-05,
        "degrees": 0.003754483732472026,
        "percentage": 0.006552810293799388,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.055066919475219946,
        "degrees": 3.155102076716861,
        "percentage": 5.5122648083149715,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.609938697019,
        "knots": 249.9976576765682,
        "feetPerMinute": 25316.91787648367
      },
      "trueAirSpeed": {
        "metersPerSecond": 150.76583839698318,
        "knots": 293.06527037294535,
        "feetPerMinute": 29678.315594781496
      },
      "groundSpeed": {
        "metersPerSecond": 136.98822313583383,
        "knots": 266.2837356132518,
        "feetPerMinute": 26966.186519578147
      },
      "machNumber": 0.4513856482282159,
      "verticalSpeed": {
        "metersPerSecond": 0.11506245188793295,
        "knots": 0.22366345672764712,
        "feetPerMinute": 22.650089679120356
      },
      "flightPathAngle": {
        "radians": 0.0008399439177684911,
        "degrees": 0.04812524151581801,
        "percentage": 0.08399441152969778,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -130.4787155115069,
        "knots": -253.6302682747496,
        "feetPerMinute": -25684.787339926337
      },
      "velocity_Y": {
        "metersPerSecond": 0.11506245188793295,
        "knots": 0.22366345672764712,
        "feetPerMinute": 22.650089679120356
      },
      "velocity_Z": {
        "metersPerSecond": -41.726227679725014,
        "knots": -81.10927731786738,
        "feetPerMinute": -8213.82460924494
      },
      "heading_Velocity": {
        "radiansPerSecond": 4.6910200160511754E-06,
        "degreesPerSecond": 0.00026877564853112405
      },
      "bank_Velocity": {
        "radiansPerSecond": 9.794616998339678E-05,
        "degreesPerSecond": 0.005611902159519584
      },
      "pitch_Velocity": {
        "radiansPerSecond": -0.0023542069153008605,
        "degreesPerSecond": -0.13488612034725178
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": -0.01207820645166302,
        "knotsPerSecond": -0.02347814914182645
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102626.3203125,
        "hectopascals": 1026.263203125,
        "inchesOfMercury": 30.309013677643236
      },
      "windDirection": {
        "angle": {
          "radians": 3.044550519004474,
          "degrees": 174.4398952533207,
          "percentage": -9.734790739339564,
          "degMinSec": {}
        },
        "radians": 3.044550519004474,
        "degrees": 174.4398952533207
      },
      "windSpeed": {
        "metersPerSecond": 13.838975659206632,
        "knots": 26.900809801294855,
        "feetPerMinute": 2724.207894105089
      },
      "windXComp": {
        "metersPerSecond": 1.3017545897202911,
        "knots": 2.5304078487002495,
        "feetPerMinute": 256.2509116882752
      },
      "windHComp": {
        "metersPerSecond": 13.777615261149363,
        "knots": 26.78153475969362,
        "feetPerMinute": 2712.1290752033565
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9468411192069238,
            "degrees": 54.25,
            "percentage": 138.908762759763,
            "degMinSec": {}
          },
          "radians": 0.9468411192069238,
          "degrees": 54.25,
          "degMinSec": {},
          "vrc": "N054.15.000.000",
          "nats": "541500N"
        },
        "lon": {
          "angle": {
            "radians": -0.09599310885968748,
            "degrees": -5.499999999999964,
            "percentage": -9.628904819753796,
            "degMinSec": {}
          },
          "radians": -0.09599310885968748,
          "degrees": -5.499999999999964,
          "degMinSec": {},
          "vrc": "E005.29.060.000",
          "nats": "0053000W"
        },
        "geoPotentialHeight": {
          "meters": 3163.129150390625,
          "feet": 10377.720641767579,
          "nauticalMiles": 1.7079531049625405,
          "statuteMiles": 1.9654773313788878
        },
        "levelPressure": {
          "pascals": 70000,
          "hectopascals": 700,
          "inchesOfMercury": 20.67336089781453
        },
        "temp": {
          "kelvin": 277.6199645996094,
          "celsius": 4.469964599609398
        },
        "v": {
          "metersPerSecond": 13.77386474609375,
          "knots": 26.77424434350586,
          "feetPerMinute": 2711.390784814453
        },
        "u": {
          "metersPerSecond": -1.340856909751892,
          "knots": -2.606416658879757,
          "feetPerMinute": -263.9482190274239
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102626.3203125,
          "hectopascals": 1026.263203125,
          "inchesOfMercury": 30.309013677643236
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9470457077023122,
            "degrees": 54.26172205732269,
            "percentage": 138.9687153166313,
            "degMinSec": {}
          },
          "radians": 0.9470457077023122,
          "degrees": 54.26172205732269,
          "degMinSec": {},
          "vrc": "N054.15.042.199",
          "nats": "541542N"
        },
        "lon": {
          "angle": {
            "radians": -0.09520870863877029,
            "degrees": -5.455057177892279,
            "percentage": -9.549743497410965,
            "degMinSec": {}
          },
          "radians": -0.09520870863877029,
          "degrees": -5.455057177892279,
          "degMinSec": {},
          "vrc": "E005.27.018.206",
          "nats": "0052718W"
        },
        "alt": {
          "meters": 3166.8875253605243,
          "feet": 10390.051268703823,
          "nauticalMiles": 1.7099824650974753,
          "statuteMiles": 1.967812677314809
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 9.794616998339678E-05,
        "degreesPerSecond": 0.005611902159519584
      },
      "pitchRate": {
        "radiansPerSecond": -0.0023542069153008605,
        "degreesPerSecond": -0.13488612034725178
      },
      "yawRate": {
        "radiansPerSecond": 4.6910200160511754E-06,
        "degreesPerSecond": 0.00026877564853112405
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 32.754424256338716,
      "thrustLeverVel": 0.034753632993073325,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 30000,
      "departureAirport": {
        "identifier": "EPKK",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.8740221043430565,
              "degrees": 50.07777778,
              "percentage": 119.50443915133857,
              "degMinSec": {}
            },
            "radians": 0.8740221043430565,
            "degrees": 50.07777778,
            "degMinSec": {},
            "vrc": "N050.04.040.000",
            "nats": "500440N"
          },
          "lon": {
            "angle": {
              "radians": 0.3453085443314823,
              "degrees": 19.78472222000002,
              "percentage": 35.97209726608166,
              "degMinSec": {}
            },
            "radians": 0.3453085443314823,
            "degrees": 19.78472222000002,
            "degMinSec": {},
            "vrc": "E019.47.005.000",
            "nats": "0194705E"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "legType": "TRACK_TO_FIX",
        "initialTrueCourse": {
          "angle": {
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336,
            "percentage": 0.5871603432202414,
            "degMinSec": {}
          },
          "radians": 3.1474641895474593,
          "degrees": 180.3364142296336
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664,
            "percentage": -11.799044018857247,
            "degMinSec": {}
          },
          "radians": 6.165737882555982,
          "degrees": 353.27075825438664
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "percentage": -3436.7770818435856,
                  "degMinSec": {}
                },
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "degMinSec": {},
                "vrc": "N088.19.060.000",
                "nats": "882000S"
              },
              "lon": {
                "angle": {
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "percentage": 1.745506492821751,
                  "degMinSec": {}
                },
                "radians": -3.12413936106985,
                "degrees": -179,
                "degMinSec": {},
                "vrc": "E179.00.000.000",
                "nats": "1790000W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "P6"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "IPSET"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "legLength": {
          "meters": 16253794.83622146,
          "feet": 53326100.25046881,
          "nauticalMiles": 8776.347103791286,
          "statuteMiles": 10099.63987576395
        },
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "percentage": -3436.7770818435856,
                  "degMinSec": {}
                },
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "degMinSec": {},
                "vrc": "N088.19.060.000",
                "nats": "882000S"
              },
              "lon": {
                "angle": {
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "percentage": 1.745506492821751,
                  "degMinSec": {}
                },
                "radians": -3.12413936106985,
                "degrees": -179,
                "degMinSec": {},
                "vrc": "E179.00.000.000",
                "nats": "1790000W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 16225436.20746342,
      "crossTrackDistance_m": 0.059557648906877335,
      "requiredTrueCourse": 180.33374321230292,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "ALT",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY879H",
    "delayMs": 420000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A320",
      "filedTas": 360,
      "origin": "EGKK",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 32000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "LAM L10 BPK UN601 LESTA UP6 RODOL UM65 TENSO L603 BELOX DCT REMSI DCT UVPOK M147 ROBOP M146 IPSET P6 BELZU DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9425802489761963,
          "degrees": 54.005870118726385,
          "percentage": 137.66785047522515,
          "degMinSec": {}
        },
        "radians": 0.9425802489761963,
        "degrees": 54.005870118726385,
        "degMinSec": {},
        "vrc": "N054.00.021.132",
        "nats": "540021N"
      },
      "longitude": {
        "angle": {
          "radians": -0.095253024150038,
          "degrees": -5.457596269654883,
          "percentage": -9.55421548214927,
          "degMinSec": {}
        },
        "radians": -0.095253024150038,
        "degrees": -5.457596269654883,
        "degMinSec": {},
        "vrc": "E005.27.027.347",
        "nats": "0052727W"
      },
      "indicatedAltitude": {
        "meters": 3047.9264784416227,
        "feet": 9999.759107530414,
        "nauticalMiles": 1.645748638467399,
        "statuteMiles": 1.8938937097610098
      },
      "trueAltitude": {
        "meters": 3175.9903415666226,
        "feet": 10419.916152225438,
        "nauticalMiles": 1.7148975926385652,
        "statuteMiles": 1.9734689050735097
      },
      "pressureAltitude": {
        "meters": 3047.9264784416227,
        "feet": 9999.759107530414,
        "nauticalMiles": 1.645748638467399,
        "statuteMiles": 1.8938937097610098
      },
      "densityAltitude": {
        "meters": 3333.3288476700855,
        "feet": 10936.118616589923,
        "nauticalMiles": 1.7998535894546899,
        "statuteMiles": 2.0712345201958597
      },
      "heading_Mag": {
        "angle": {
          "radians": 3.1594258088639453,
          "degrees": 181.02176453261038,
          "percentage": 1.783504595649302,
          "degMinSec": {}
        },
        "radians": 3.1594258088639453,
        "degrees": 181.02176453261038
      },
      "heading_True": {
        "angle": {
          "radians": 3.1381524586584053,
          "degrees": 179.80289134972918,
          "percentage": -0.34402085029537155,
          "degMinSec": {}
        },
        "radians": 3.1381524586584053,
        "degrees": 179.80289134972918
      },
      "track_True": {
        "angle": {
          "radians": 3.1473895432167573,
          "degrees": 180.3321373099282,
          "percentage": 0.5796954560593404,
          "degMinSec": {}
        },
        "radians": 3.1473895432167573,
        "degrees": 180.3321373099282
      },
      "track_Mag": {
        "angle": {
          "radians": 3.1686628934222973,
          "degrees": 181.5510104928094,
          "percentage": 2.707685410944366,
          "degMinSec": {}
        },
        "radians": 3.1686628934222973,
        "degrees": 181.5510104928094
      },
      "bank": {
        "radians": 9.96242763711874E-07,
        "degrees": 5.70805057311393E-05,
        "percentage": 9.962427637122034E-05,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.055503159442965855,
        "degrees": 3.1800967857236246,
        "percentage": 5.5560224118818065,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61064536052083,
        "knots": 249.99903132017624,
        "feetPerMinute": 25317.056983476672
      },
      "trueAirSpeed": {
        "metersPerSecond": 150.73934340918902,
        "knots": 293.0137682498916,
        "feetPerMinute": 29673.10004583622
      },
      "groundSpeed": {
        "metersPerSecond": 136.66060493501752,
        "knots": 265.64689693930416,
        "feetPerMinute": 26901.694745700173
      },
      "machNumber": 0.45162141293007946,
      "verticalSpeed": {
        "metersPerSecond": 0.008100909523734904,
        "knots": 0.01574690437225495,
        "feetPerMinute": 1.5946672801110253
      },
      "flightPathAngle": {
        "radians": 5.927757687080706E-05,
        "degrees": 0.00339635497445955,
        "percentage": 0.005927757694023752,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -130.169796172799,
        "knots": -253.0297772717183,
        "feetPerMinute": -25623.976444533953
      },
      "velocity_Y": {
        "metersPerSecond": 0.008100909523734904,
        "knots": 0.01574690437225495,
        "feetPerMinute": 1.5946672801110253
      },
      "velocity_Z": {
        "metersPerSecond": -41.61664457325816,
        "knots": -80.89626485386043,
        "feetPerMinute": -8192.253130903699
      },
      "heading_Velocity": {
        "radiansPerSecond": 7.148937302574111E-08,
        "degreesPerSecond": 4.096039354411358E-06
      },
      "bank_Velocity": {
        "radiansPerSecond": 6.480856204832845E-07,
        "degreesPerSecond": 3.713257081680942E-05
      },
      "pitch_Velocity": {
        "radiansPerSecond": 4.803117816740143E-06,
        "degreesPerSecond": 0.0002751983794033006
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.005350528371799723,
        "knotsPerSecond": 0.01040059247235266
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102725.5234375,
        "hectopascals": 1027.255234375,
        "inchesOfMercury": 30.338311706290607
      },
      "windDirection": {
        "angle": {
          "radians": 3.039569908096544,
          "degrees": 174.15452726889947,
          "percentage": -10.23781981752076,
          "degMinSec": {}
        },
        "radians": 3.039569908096544,
        "degrees": 174.15452726889947
      },
      "windSpeed": {
        "metersPerSecond": 14.147428846372723,
        "knots": 27.500394678448536,
        "feetPerMinute": 2784.927027380009
      },
      "windXComp": {
        "metersPerSecond": 1.3924316640546026,
        "knots": 2.7066699355825548,
        "feetPerMinute": 274.10073004181413
      },
      "windHComp": {
        "metersPerSecond": 14.078738474171512,
        "knots": 27.36687131058745,
        "feetPerMinute": 2771.405300136052
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.09599310885968748,
            "degrees": -5.499999999999964,
            "percentage": -9.628904819753796,
            "degMinSec": {}
          },
          "radians": -0.09599310885968748,
          "degrees": -5.499999999999964,
          "degMinSec": {},
          "vrc": "E005.29.060.000",
          "nats": "0053000W"
        },
        "geoPotentialHeight": {
          "meters": 3163.44921875,
          "feet": 10378.77073484375,
          "nauticalMiles": 1.708125928050756,
          "statuteMiles": 1.9656762126369502
        },
        "levelPressure": {
          "pascals": 70000,
          "hectopascals": 700,
          "inchesOfMercury": 20.67336089781453
        },
        "temp": {
          "kelvin": 277.28997802734375,
          "celsius": 4.139978027343773
        },
        "v": {
          "metersPerSecond": 14.073864936828613,
          "knots": 27.357397914264677,
          "feetPerMinute": 2770.4459423606872
        },
        "u": {
          "metersPerSecond": -1.44085693359375,
          "knots": -2.8008011052246093,
          "feetPerMinute": -283.6332637207031
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102725.5234375,
          "hectopascals": 1027.255234375,
          "inchesOfMercury": 30.338311706290607
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9425802489761963,
            "degrees": 54.005870118726385,
            "percentage": 137.66785047522515,
            "degMinSec": {}
          },
          "radians": 0.9425802489761963,
          "degrees": 54.005870118726385,
          "degMinSec": {},
          "vrc": "N054.00.021.132",
          "nats": "540021N"
        },
        "lon": {
          "angle": {
            "radians": -0.095253024150038,
            "degrees": -5.457596269654883,
            "percentage": -9.55421548214927,
            "degMinSec": {}
          },
          "radians": -0.095253024150038,
          "degrees": -5.457596269654883,
          "degMinSec": {},
          "vrc": "E005.27.027.347",
          "nats": "0052727W"
        },
        "alt": {
          "meters": 3175.9903415666226,
          "feet": 10419.916152225438,
          "nauticalMiles": 1.7148975926385652,
          "statuteMiles": 1.9734689050735097
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 6.480856204832845E-07,
        "degreesPerSecond": 3.713257081680942E-05
      },
      "pitchRate": {
        "radiansPerSecond": 4.803117816740143E-06,
        "degreesPerSecond": 0.0002751983794033006
      },
      "yawRate": {
        "radiansPerSecond": 7.148937302574111E-08,
        "degreesPerSecond": 4.096039354411358E-06
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 32.72716356016502,
      "thrustLeverVel": 0.015009461469674079,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 32000,
      "departureAirport": {
        "identifier": "EGKK",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.8927019755149921,
              "degrees": 51.14805556,
              "percentage": 124.14427272614329,
              "degMinSec": {}
            },
            "radians": 0.8927019755149921,
            "degrees": 51.14805556,
            "degMinSec": {},
            "vrc": "N051.08.053.000",
            "nats": "510853N"
          },
          "lon": {
            "angle": {
              "radians": -0.003320973754386003,
              "degrees": -0.19027778000003365,
              "percentage": -0.33209859632987887,
              "degMinSec": {}
            },
            "radians": -0.003320973754386003,
            "degrees": -0.19027778000003365,
            "degMinSec": {},
            "vrc": "E000.11.025.000",
            "nats": "0001125E"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "legType": "TRACK_TO_FIX",
        "initialTrueCourse": {
          "angle": {
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336,
            "percentage": 0.5871603432202414,
            "degMinSec": {}
          },
          "radians": 3.1474641895474593,
          "degrees": 180.3364142296336
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664,
            "percentage": -11.799044018857247,
            "degMinSec": {}
          },
          "radians": 6.165737882555982,
          "degrees": 353.27075825438664
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "percentage": -3436.7770818435856,
                  "degMinSec": {}
                },
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "degMinSec": {},
                "vrc": "N088.19.060.000",
                "nats": "882000S"
              },
              "lon": {
                "angle": {
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "percentage": 1.745506492821751,
                  "degMinSec": {}
                },
                "radians": -3.12413936106985,
                "degrees": -179,
                "degMinSec": {},
                "vrc": "E179.00.000.000",
                "nats": "1790000W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "P6"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "IPSET"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "legLength": {
          "meters": 16253794.83622146,
          "feet": 53326100.25046881,
          "nauticalMiles": 8776.347103791286,
          "statuteMiles": 10099.63987576395
        },
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "percentage": -3436.7770818435856,
                  "degMinSec": {}
                },
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "degMinSec": {},
                "vrc": "N088.19.060.000",
                "nats": "882000S"
              },
              "lon": {
                "angle": {
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "percentage": 1.745506492821751,
                  "degMinSec": {}
                },
                "radians": -3.12413936106985,
                "degrees": -179,
                "degMinSec": {},
                "vrc": "E179.00.000.000",
                "nats": "1790000W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 16197013.21121034,
      "crossTrackDistance_m": -0.0205471941118244,
      "requiredTrueCourse": 180.33169479739522,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "ALT",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY747V",
    "delayMs": 1080000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A320",
      "filedTas": 440,
      "origin": "EGCC",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 24000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "EKLAD Y53 WAL M146 IPSET P6 BELZU DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9502761119220579,
          "degrees": 54.44681058523537,
          "percentage": 139.91989460817913,
          "degMinSec": {}
        },
        "radians": 0.9502761119220579,
        "degrees": 54.44681058523537,
        "degMinSec": {},
        "vrc": "N054.26.048.518",
        "nats": "542649N"
      },
      "longitude": {
        "angle": {
          "radians": -0.09351803831344618,
          "degrees": -5.358188903703198,
          "percentage": -9.379161992757293,
          "degMinSec": {}
        },
        "radians": -0.09351803831344618,
        "degrees": -5.358188903703198,
        "degMinSec": {},
        "vrc": "E005.21.029.480",
        "nats": "0052129W"
      },
      "indicatedAltitude": {
        "meters": 3047.9131399974067,
        "feet": 9999.715346229092,
        "nauticalMiles": 1.6457414362836968,
        "statuteMiles": 1.8938854216360248
      },
      "trueAltitude": {
        "meters": 3176.562076247407,
        "feet": 10421.791922235541,
        "nauticalMiles": 1.715206304669226,
        "statuteMiles": 1.973824164533752
      },
      "pressureAltitude": {
        "meters": 3047.9131399974067,
        "feet": 9999.715346229092,
        "nauticalMiles": 1.6457414362836968,
        "statuteMiles": 1.8938854216360248
      },
      "densityAltitude": {
        "meters": 3315.8215383221836,
        "feet": 10878.679935788952,
        "nauticalMiles": 1.7904003986620862,
        "statuteMiles": 2.060355982513486
      },
      "heading_Mag": {
        "angle": {
          "radians": 4.6797636988640505,
          "degrees": 268.1307090634412,
          "percentage": 3064.0200642864042,
          "degMinSec": {}
        },
        "radians": 4.6797636988640505,
        "degrees": 268.1307090634412
      },
      "heading_True": {
        "angle": {
          "radians": 4.6581101319290426,
          "degrees": 266.8900510666612,
          "percentage": 1840.528623979264,
          "degMinSec": {}
        },
        "radians": 4.6581101319290426,
        "degrees": 266.8900510666612
      },
      "track_True": {
        "angle": {
          "radians": 4.7408591503327635,
          "degrees": 271.63122058004484,
          "percentage": -3511.499241826848,
          "degMinSec": {}
        },
        "radians": 4.7408591503327635,
        "degrees": 271.63122058004484
      },
      "track_Mag": {
        "angle": {
          "radians": 4.7625127172677715,
          "degrees": 272.8718785768248,
          "percentage": -1993.39167195077,
          "degMinSec": {}
        },
        "radians": 4.7625127172677715,
        "degrees": 272.8718785768248
      },
      "bank": {
        "radians": -0.2706797874183248,
        "degrees": -15.508809418568333,
        "percentage": -27.749012948298862,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.055458356822826675,
        "degrees": 3.177529784678522,
        "percentage": 5.5515283307584244,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.60516217363994,
        "knots": 249.98837286025693,
        "feetPerMinute": 25315.977615945892
      },
      "trueAirSpeed": {
        "metersPerSecond": 150.59684473702845,
        "knots": 292.7367730610043,
        "feetPerMinute": 29645.049125221947
      },
      "groundSpeed": {
        "metersPerSecond": 150.56002320339834,
        "knots": 292.6651977437866,
        "feetPerMinute": 29637.80079159824
      },
      "machNumber": 0.45156088488226426,
      "verticalSpeed": {
        "metersPerSecond": -0.0013284248960833103,
        "knots": -0.002582250763702166,
        "feetPerMinute": -0.26150097216395807
      },
      "flightPathAngle": {
        "radians": -8.823224570404116E-06,
        "degrees": -0.0005055335295802847,
        "percentage": -0.0008823224570633075,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 149.5386897376122,
        "knots": 290.67988481431905,
        "feetPerMinute": 29436.75089032486
      },
      "velocity_Y": {
        "metersPerSecond": -0.0013284248960833103,
        "knots": -0.002582250763702166,
        "feetPerMinute": -0.26150097216395807
      },
      "velocity_Z": {
        "metersPerSecond": 17.507165920445246,
        "knots": 34.031199431461964,
        "feetPerMinute": 3446.292614306015
      },
      "heading_Velocity": {
        "radiansPerSecond": -0.018068748884984037,
        "degreesPerSecond": -1.0352630521912973
      },
      "bank_Velocity": {
        "radiansPerSecond": -0.08726646259971647,
        "degreesPerSecond": -5
      },
      "pitch_Velocity": {
        "radiansPerSecond": 8.756741252892442E-05,
        "degreesPerSecond": 0.005017243160788376
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.0016909481609295174,
        "knotsPerSecond": 0.0032869394369338767
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102731.921875,
        "hectopascals": 1027.31921875,
        "inchesOfMercury": 30.340201380685173
      },
      "windDirection": {
        "angle": {
          "radians": 3.090261815742311,
          "degrees": 177.05895963246888,
          "percentage": -5.137596851663482,
          "degMinSec": {}
        },
        "radians": 3.090261815742311,
        "degrees": 177.05895963246888
      },
      "windSpeed": {
        "metersPerSecond": 12.490316982827302,
        "knots": 24.279227725166955,
        "feetPerMinute": 2458.723894196348
      },
      "windXComp": {
        "metersPerSecond": 12.49026270765212,
        "knots": 24.279122222693324,
        "feetPerMinute": 2458.713210106403
      },
      "windHComp": {
        "metersPerSecond": 0.036821533630105804,
        "knots": 0.07157531721767939,
        "feetPerMinute": 7.24833362369978
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9512044423369096,
            "degrees": 54.5,
            "percentage": 140.19482944763357,
            "degMinSec": {}
          },
          "radians": 0.9512044423369096,
          "degrees": 54.5,
          "degMinSec": {},
          "vrc": "N054.30.000.000",
          "nats": "543000N"
        },
        "lon": {
          "angle": {
            "radians": -0.0916292530974836,
            "degrees": -5.249969482421836,
            "percentage": -9.188655453857013,
            "degMinSec": {}
          },
          "radians": -0.0916292530974836,
          "degrees": -5.249969482421836,
          "degMinSec": {},
          "vrc": "E005.14.059.890",
          "nats": "0051500W"
        },
        "geoPotentialHeight": {
          "meters": 3165.609130859375,
          "feet": 10385.857060888671,
          "nauticalMiles": 1.7092921872890794,
          "statuteMiles": 1.9670183197994804
        },
        "levelPressure": {
          "pascals": 70000,
          "hectopascals": 700,
          "inchesOfMercury": 20.67336089781453
        },
        "temp": {
          "kelvin": 276.8299865722656,
          "celsius": 3.6799865722656477
        },
        "v": {
          "metersPerSecond": 12.473865509033203,
          "knots": 24.247248626541136,
          "feetPerMinute": 2455.48541499939
        },
        "u": {
          "metersPerSecond": -0.640856921672821,
          "knots": -1.245725882052183,
          "feetPerMinute": -126.15294137406349
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102731.921875,
          "hectopascals": 1027.31921875,
          "inchesOfMercury": 30.340201380685173
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9502761119220579,
            "degrees": 54.44681058523537,
            "percentage": 139.91989460817913,
            "degMinSec": {}
          },
          "radians": 0.9502761119220579,
          "degrees": 54.44681058523537,
          "degMinSec": {},
          "vrc": "N054.26.048.518",
          "nats": "542649N"
        },
        "lon": {
          "angle": {
            "radians": -0.09351803831344618,
            "degrees": -5.358188903703198,
            "percentage": -9.379161992757293,
            "degMinSec": {}
          },
          "radians": -0.09351803831344618,
          "degrees": -5.358188903703198,
          "degMinSec": {},
          "vrc": "E005.21.029.480",
          "nats": "0052129W"
        },
        "alt": {
          "meters": 3176.562076247407,
          "feet": 10421.791922235541,
          "nauticalMiles": 1.715206304669226,
          "statuteMiles": 1.973824164533752
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": -0.08726646259971647,
        "degreesPerSecond": -5
      },
      "pitchRate": {
        "radiansPerSecond": 8.756741252892442E-05,
        "degreesPerSecond": 0.005017243160788376
      },
      "yawRate": {
        "radiansPerSecond": -0.018068748884984037,
        "degreesPerSecond": -1.0352630521912973
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 32.489656060874715,
      "thrustLeverVel": 0.20635521595670525,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 24000,
      "departureAirport": {
        "identifier": "EGCC",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9311961816981266,
              "degrees": 53.35361111,
              "percentage": 134.42260550039376,
              "degMinSec": {}
            },
            "radians": 0.9311961816981266,
            "degrees": 53.35361111,
            "degMinSec": {},
            "vrc": "N053.21.013.000",
            "nats": "532113N"
          },
          "lon": {
            "angle": {
              "radians": -0.039706240482871635,
              "degrees": -2.2750000000000368,
              "percentage": -3.972712041201031,
              "degMinSec": {}
            },
            "radians": -0.039706240482871635,
            "degrees": -2.2750000000000368,
            "degMinSec": {},
            "vrc": "E002.16.030.000",
            "nats": "0021630W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "legType": "TRACK_TO_FIX",
        "initialTrueCourse": {
          "angle": {
            "radians": 5.460531278681644,
            "degrees": 312.8653961676329,
            "percentage": -107.74323687641969,
            "degMinSec": {}
          },
          "radians": 5.460531278681644,
          "degrees": 312.8653961676329
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.447293429813374,
            "degrees": 312.1069232976491,
            "percentage": -110.64531286515107,
            "degMinSec": {}
          },
          "radians": 5.447293429813374,
          "degrees": 312.1069232976491
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "IPSET"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPEG"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "legLength": {
          "meters": 82274.60605599961,
          "feet": 269929.8185327658,
          "nauticalMiles": 44.424733291576466,
          "statuteMiles": 51.123070055873455
        },
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.1474641895474593,
              "degrees": 180.3364142296336,
              "percentage": 0.5871603432202414,
              "degMinSec": {}
            },
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 6.165737882555982,
              "degrees": 353.27075825438664,
              "percentage": -11.799044018857247,
              "degMinSec": {}
            },
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16253794.83622146,
            "feet": 53326100.25046881,
            "nauticalMiles": 8776.347103791286,
            "statuteMiles": 10099.63987576395
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PEPEG =(TF)=> IPSET; IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 10030.202852298695,
      "crossTrackDistance_m": -1646.8123339074807,
      "requiredTrueCourse": 312.20058647531243,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "ALT",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "RYR2KZ",
    "delayMs": 1200000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "B38M",
      "filedTas": 460,
      "origin": "LEMG",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 38000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "DOLUR UNFIT DCT PENWU DCT NICXI DCT VATRY DCT DUB N34 BELZU DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.942859127832368,
          "degrees": 54.021848700180456,
          "percentage": 137.74862371359936,
          "degMinSec": {}
        },
        "radians": 0.942859127832368,
        "degrees": 54.021848700180456,
        "degMinSec": {},
        "vrc": "N054.01.018.655",
        "nats": "540119N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10948401065165037,
          "degrees": -6.272971734504916,
          "percentage": -10.99235707673613,
          "degMinSec": {}
        },
        "radians": -0.10948401065165037,
        "degrees": -6.272971734504916,
        "degMinSec": {},
        "vrc": "E006.16.022.698",
        "nats": "0061623W"
      },
      "indicatedAltitude": {
        "meters": 3623.6550097454915,
        "feet": 11888.632302173399,
        "nauticalMiles": 1.9566171758884943,
        "statuteMiles": 2.251634833662344
      },
      "trueAltitude": {
        "meters": 3674.6156647454914,
        "feet": 12055.826057523598,
        "nauticalMiles": 1.9841337282643041,
        "statuteMiles": 2.2833003166168893
      },
      "pressureAltitude": {
        "meters": 3623.6550097454915,
        "feet": 11888.632302173399,
        "nauticalMiles": 1.9566171758884943,
        "statuteMiles": 2.251634833662344
      },
      "densityAltitude": {
        "meters": 3842.776629598049,
        "feet": 12607.535277450463,
        "nauticalMiles": 2.0749333853121215,
        "statuteMiles": 2.3877906958351036
      },
      "heading_Mag": {
        "angle": {
          "radians": 0.0787652816821014,
          "degrees": 4.512918212543504,
          "percentage": 7.892857271945192,
          "degMinSec": {}
        },
        "radians": 0.0787652816821014,
        "degrees": 4.512918212543504
      },
      "heading_True": {
        "angle": {
          "radians": 0.052005207935375886,
          "degrees": 2.979678927397296,
          "percentage": 5.205214212722159,
          "degMinSec": {}
        },
        "radians": 0.052005207935375886,
        "degrees": 2.979678927397296
      },
      "track_True": {
        "angle": {
          "radians": 0.039661758739042874,
          "degrees": 2.2724513838132663,
          "percentage": 3.968256854383159,
          "degMinSec": {}
        },
        "radians": 0.039661758739042874,
        "degrees": 2.2724513838132663
      },
      "track_Mag": {
        "angle": {
          "radians": 0.06642183248576838,
          "degrees": 3.8056906689594743,
          "percentage": 6.651968644812668,
          "degMinSec": {}
        },
        "radians": 0.06642183248576838,
        "degrees": 3.8056906689594743
      },
      "bank": {
        "radians": -0.000255033549418933,
        "degrees": -0.014612346015945969,
        "percentage": -0.025503355494823996,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.03487206922587116,
        "degrees": -1.9980223895304574,
        "percentage": -3.488621162871236,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 149.18963948272145,
        "knots": 290.00138557065117,
        "feetPerMinute": 29368.04020802951
      },
      "trueAirSpeed": {
        "metersPerSecond": 178.795399662263,
        "knots": 347.55036486109196,
        "feetPerMinute": 35195.945941676335
      },
      "groundSpeed": {
        "metersPerSecond": 195.60937411204927,
        "knots": 380.2341082114623,
        "feetPerMinute": 38505.78353770654
      },
      "machNumber": 0.5382106922996736,
      "verticalSpeed": {
        "metersPerSecond": -11.306811433442677,
        "knots": -21.978677564028946,
        "feetPerMinute": -2225.7503533977642
      },
      "flightPathAngle": {
        "radians": -0.0577387660492323,
        "degrees": -3.3081876089142574,
        "percentage": -5.7803014220401785,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 149.40153469623448,
        "knots": 290.4132768100672,
        "feetPerMinute": 29409.751865567632
      },
      "velocity_Y": {
        "metersPerSecond": -11.306811433442677,
        "knots": -21.978677564028946,
        "feetPerMinute": -2225.7503533977642
      },
      "velocity_Z": {
        "metersPerSecond": -126.26245946803627,
        "knots": -245.43452426218548,
        "feetPerMinute": -24854.815651266726
      },
      "heading_Velocity": {
        "radiansPerSecond": -1.2786424567345357E-05,
        "degreesPerSecond": -0.0007326081627712786
      },
      "bank_Velocity": {
        "radiansPerSecond": 0.0003419504379277793,
        "degreesPerSecond": 0.019592316895911985
      },
      "pitch_Velocity": {
        "radiansPerSecond": -0.0001150527133847723,
        "degreesPerSecond": -0.006592034898475769
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": -0.0005819417674052252,
        "knotsPerSecond": -0.0011312040129200424
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 101882.3125,
        "hectopascals": 1018.823125,
        "inchesOfMercury": 30.089283077377438
      },
      "windDirection": {
        "angle": {
          "radians": 3.0630803881947344,
          "degrees": 175.50157855285212,
          "percentage": -7.867398529062189,
          "degMinSec": {}
        },
        "radians": 3.0630803881947344,
        "degrees": 175.50157855285212
      },
      "windSpeed": {
        "metersPerSecond": 16.958209469202334,
        "knots": 32.96411372745214,
        "feetPerMinute": 3338.230317296267
      },
      "windXComp": {
        "metersPerSecond": -2.2070640233756,
        "knots": -4.2901881594545195,
        "feetPerMinute": -434.4614358270962
      },
      "windHComp": {
        "metersPerSecond": -16.81397444978625,
        "knots": -32.68374335037031,
        "feetPerMinute": -3309.8375960302033
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.10908254561742758,
            "degrees": -6.249969482421875,
            "percentage": -10.95172726625836,
            "degMinSec": {}
          },
          "radians": -0.10908254561742758,
          "degrees": -6.249969482421875,
          "degMinSec": {},
          "vrc": "E006.14.059.890",
          "nats": "0061500W"
        },
        "geoPotentialHeight": {
          "meters": 3754.75390625,
          "feet": 12318.74680578125,
          "nauticalMiles": 2.0274049169816415,
          "statuteMiles": 2.3330959112843495
        },
        "levelPressure": {
          "pascals": 65000,
          "hectopascals": 650,
          "inchesOfMercury": 19.19669226225635
        },
        "temp": {
          "kelvin": 274.1000061035156,
          "celsius": 0.9500061035156477
        },
        "v": {
          "metersPerSecond": 16.905969619750977,
          "knots": 32.86256760953522,
          "feetPerMinute": 3327.9468820358275
        },
        "u": {
          "metersPerSecond": -1.3300600051879883,
          "knots": -2.58542916072464,
          "feetPerMinute": -261.82284404525757
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 101882.3125,
          "hectopascals": 1018.823125,
          "inchesOfMercury": 30.089283077377438
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.942859127832368,
            "degrees": 54.021848700180456,
            "percentage": 137.74862371359936,
            "degMinSec": {}
          },
          "radians": 0.942859127832368,
          "degrees": 54.021848700180456,
          "degMinSec": {},
          "vrc": "N054.01.018.655",
          "nats": "540119N"
        },
        "lon": {
          "angle": {
            "radians": -0.10948401065165037,
            "degrees": -6.272971734504916,
            "percentage": -10.99235707673613,
            "degMinSec": {}
          },
          "radians": -0.10948401065165037,
          "degrees": -6.272971734504916,
          "degMinSec": {},
          "vrc": "E006.16.022.698",
          "nats": "0061623W"
        },
        "alt": {
          "meters": 3674.6156647454914,
          "feet": 12055.826057523598,
          "nauticalMiles": 1.9841337282643041,
          "statuteMiles": 2.2833003166168893
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0.0003419504379277793,
        "degreesPerSecond": 0.019592316895911985
      },
      "pitchRate": {
        "radiansPerSecond": -0.0001150527133847723,
        "degreesPerSecond": -0.006592034898475769
      },
      "yawRate": {
        "radiansPerSecond": -1.2786424567345357E-05,
        "degreesPerSecond": -0.0007326081627712786
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 38000,
      "departureAirport": {
        "identifier": "LEMG",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.6400995031689203,
              "degrees": 36.675,
              "percentage": 74.46984959835642,
              "degMinSec": {}
            },
            "radians": 0.6400995031689203,
            "degrees": 36.675,
            "degMinSec": {},
            "vrc": "N036.40.030.000",
            "nats": "364030N"
          },
          "lon": {
            "angle": {
              "radians": -0.07852527198748938,
              "degrees": -4.499166670000011,
              "percentage": -7.868707240199692,
              "degMinSec": {}
            },
            "radians": -0.07852527198748938,
            "degrees": -4.499166670000011,
            "degMinSec": {},
            "vrc": "E004.29.057.000",
            "nats": "0042957W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9337404104141093,
                  "degrees": 53.499384677541805,
                  "percentage": 135.1392085154476,
                  "degMinSec": {}
                },
                "radians": 0.9337404104141093,
                "degrees": 53.499384677541805,
                "degMinSec": {},
                "vrc": "N053.29.057.785",
                "nats": "532958N"
              },
              "lon": {
                "angle": {
                  "radians": -0.11007662415653385,
                  "degrees": -6.306925987217195,
                  "percentage": -11.052338407802706,
                  "degMinSec": {}
                },
                "radians": -0.11007662415653385,
                "degrees": -6.306925987217195,
                "degMinSec": {},
                "vrc": "E006.18.024.934",
                "nats": "0061825W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9436705832347494,
                  "degrees": 54.06834167,
                  "percentage": 137.98400349865105,
                  "degMinSec": {}
                },
                "radians": 0.9436705832347494,
                "degrees": 54.06834167,
                "degMinSec": {},
                "vrc": "N054.04.006.030",
                "nats": "540406N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10942933220008655,
                  "degrees": -6.269838889999999,
                  "percentage": -10.986823195815726,
                  "degMinSec": {}
                },
                "radians": -0.10942933220008655,
                "degrees": -6.269838889999999,
                "degMinSec": {},
                "vrc": "E006.16.011.420",
                "nats": "0061611W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "NEVRI"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 0.03823308121419888,
            "degrees": 2.190594191354509,
            "percentage": 3.825172141792102,
            "degMinSec": {}
          },
          "radians": 0.03823308121419888,
          "degrees": 2.190594191354509
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 0.03875531888657058,
            "degrees": 2.220516205884143,
            "percentage": 3.8774733720918926,
            "degMinSec": {}
          },
          "radians": 0.03875531888657058,
          "degrees": 2.220516205884143
        },
        "legLength": {
          "meters": 63312.03031241654,
          "feet": 207716.64153018867,
          "nauticalMiles": 34.185761507784306,
          "statuteMiles": 39.34027175819249
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9337404104141093,
                  "degrees": 53.499384677541805,
                  "percentage": 135.1392085154476,
                  "degMinSec": {}
                },
                "radians": 0.9337404104141093,
                "degrees": 53.499384677541805,
                "degMinSec": {},
                "vrc": "N053.29.057.785",
                "nats": "532958N"
              },
              "lon": {
                "angle": {
                  "radians": -0.11007662415653385,
                  "degrees": -6.306925987217195,
                  "percentage": -11.052338407802706,
                  "degMinSec": {}
                },
                "radians": -0.11007662415653385,
                "degrees": -6.306925987217195,
                "degMinSec": {},
                "vrc": "E006.18.024.934",
                "nats": "0061825W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9436705832347494,
                  "degrees": 54.06834167,
                  "percentage": 137.98400349865105,
                  "degMinSec": {}
                },
                "radians": 0.9436705832347494,
                "degrees": 54.06834167,
                "degMinSec": {},
                "vrc": "N054.04.006.030",
                "nats": "540406N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10942933220008655,
                  "degrees": -6.269838889999999,
                  "percentage": -10.986823195815726,
                  "degMinSec": {}
                },
                "radians": -0.10942933220008655,
                "degrees": -6.269838889999999,
                "degMinSec": {},
                "vrc": "E006.16.011.420",
                "nats": "0061611W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 0.03885461667627865,
              "degrees": 2.2262055501493934,
              "percentage": 3.8874181186089394,
              "degMinSec": {}
            },
            "radians": 0.03885461667627865,
            "degrees": 2.2262055501493934
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.03941974000940096,
              "degrees": 2.258584732041667,
              "percentage": 3.944017102901926,
              "degMinSec": {}
            },
            "radians": 0.03941974000940096,
            "degrees": 2.258584732041667
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9436705832347494,
                    "degrees": 54.06834167,
                    "percentage": 137.98400349865105,
                    "degMinSec": {}
                  },
                  "radians": 0.9436705832347494,
                  "degrees": 54.06834167,
                  "degMinSec": {},
                  "vrc": "N054.04.006.030",
                  "nats": "540406N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10942933220008655,
                    "degrees": -6.269838889999999,
                    "percentage": -10.986823195815726,
                    "degMinSec": {}
                  },
                  "radians": -0.10942933220008655,
                  "degrees": -6.269838889999999,
                  "degMinSec": {},
                  "vrc": "E006.16.011.420",
                  "nats": "0061611W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "NEVRI"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 65963.46315317662,
            "feet": 216415.568451468,
            "nauticalMiles": 35.61742070905865,
            "statuteMiles": 40.987795743592805
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9436705832347494,
                    "degrees": 54.06834167,
                    "percentage": 137.98400349865105,
                    "degMinSec": {}
                  },
                  "radians": 0.9436705832347494,
                  "degrees": 54.06834167,
                  "degMinSec": {},
                  "vrc": "N054.04.006.030",
                  "nats": "540406N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10942933220008655,
                    "degrees": -6.269838889999999,
                    "percentage": -10.986823195815726,
                    "degMinSec": {}
                  },
                  "radians": -0.10942933220008655,
                  "degrees": -6.269838889999999,
                  "degMinSec": {},
                  "vrc": "E006.16.011.420",
                  "nats": "0061611W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> NEVRI; NEVRI =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9337404104141093,
                "degrees": 53.499384677541805,
                "percentage": 135.1392085154476,
                "degMinSec": {}
              },
              "radians": 0.9337404104141093,
              "degrees": 53.499384677541805,
              "degMinSec": {},
              "vrc": "N053.29.057.785",
              "nats": "532958N"
            },
            "lon": {
              "angle": {
                "radians": -0.11007662415653385,
                "degrees": -6.306925987217195,
                "percentage": -11.052338407802706,
                "degMinSec": {}
              },
              "radians": -0.11007662415653385,
              "degrees": -6.306925987217195,
              "degMinSec": {},
              "vrc": "E006.18.024.934",
              "nats": "0061825W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9436705832347494,
                "degrees": 54.06834167,
                "percentage": 137.98400349865105,
                "degMinSec": {}
              },
              "radians": 0.9436705832347494,
              "degrees": 54.06834167,
              "degMinSec": {},
              "vrc": "N054.04.006.030",
              "nats": "540406N"
            },
            "lon": {
              "angle": {
                "radians": -0.10942933220008655,
                "degrees": -6.269838889999999,
                "percentage": -10.986823195815726,
                "degMinSec": {}
              },
              "radians": -0.10942933220008655,
              "degrees": -6.269838889999999,
              "degMinSec": {},
              "vrc": "E006.16.011.420",
              "nats": "0061611W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9436705832347494,
                "degrees": 54.06834167,
                "percentage": 137.98400349865105,
                "degMinSec": {}
              },
              "radians": 0.9436705832347494,
              "degrees": 54.06834167,
              "degMinSec": {},
              "vrc": "N054.04.006.030",
              "nats": "540406N"
            },
            "lon": {
              "angle": {
                "radians": -0.10942933220008655,
                "degrees": -6.269838889999999,
                "percentage": -10.986823195815726,
                "degMinSec": {}
              },
              "radians": -0.10942933220008655,
              "degrees": -6.269838889999999,
              "degMinSec": {},
              "vrc": "E006.16.011.420",
              "nats": "0061611W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 5799.887443748904,
      "crossTrackDistance_m": -4.825624484730433,
      "requiredTrueCourse": 2.217733639087917,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 5,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 0.08726646259971638,
          "degrees": 4.999999999999995,
          "percentage": 8.74886635259239,
          "degMinSec": {}
        },
        "radians": 0.08726646259971638,
        "degrees": 4.999999999999995
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "RUK56PW",
    "delayMs": 1200000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "B738",
      "filedTas": 330,
      "origin": "EGAA",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 17000,
      "destination": "EGPH",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "BLACA P600 GIRVA"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9540598847689765,
          "degrees": 54.66360480000001,
          "percentage": 141.0450049177102,
          "degMinSec": {}
        },
        "radians": 0.9540598847689765,
        "degrees": 54.66360480000001,
        "degMinSec": {},
        "vrc": "N054.39.048.977",
        "nats": "543949N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10815966817564693,
          "degrees": -6.197092500000013,
          "percentage": -10.858342031858847,
          "degMinSec": {}
        },
        "radians": -0.10815966817564693,
        "degrees": -6.197092500000013,
        "degMinSec": {},
        "vrc": "E006.11.049.533",
        "nats": "0061150W"
      },
      "indicatedAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "trueAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "pressureAltitude": {
        "meters": 67.88681551103528,
        "feet": 222.72577980122497,
        "nauticalMiles": 0.036655947900127044,
        "statuteMiles": 0.04218291149128793
      },
      "densityAltitude": {
        "meters": -72.58308962302895,
        "feet": -238.1335037588183,
        "nauticalMiles": -0.039191733057791014,
        "statuteMiles": -0.045101040935330766
      },
      "heading_Mag": {
        "angle": {
          "radians": 2.742757648739527,
          "degrees": 157.14843750000003,
          "percentage": -42.1420651362081,
          "degMinSec": {}
        },
        "radians": 2.742757648739527,
        "degrees": 157.14843750000003
      },
      "heading_True": {
        "angle": {
          "radians": 2.7150001683767773,
          "degrees": 155.5580510252972,
          "percentage": -45.450321426367985,
          "degMinSec": {}
        },
        "radians": 2.7150001683767773,
        "degrees": 155.5580510252972
      },
      "track_True": {
        "angle": {
          "radians": 2.7150001683767773,
          "degrees": 155.5580510252972,
          "percentage": -45.450321426367985,
          "degMinSec": {}
        },
        "radians": 2.7150001683767773,
        "degrees": 155.5580510252972
      },
      "track_Mag": {
        "angle": {
          "radians": 2.742757648739527,
          "degrees": 157.14843750000003,
          "percentage": -42.1420651362081,
          "degMinSec": {}
        },
        "radians": 2.742757648739527,
        "degrees": 157.14843750000003
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 6.451128390606732,
        "knots": 12.539987215310552,
        "feetPerMinute": 1269.9072041422914
      },
      "trueAirSpeed": {
        "metersPerSecond": 6.428710688905679,
        "knots": 12.49641070036517,
        "feetPerMinute": 1265.4942705953583
      },
      "groundSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "machNumber": 0.018934363403326642,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -0,
        "knots": -0,
        "feetPerMinute": -0
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101475.9140625,
        "hectopascals": 1014.759140625,
        "inchesOfMercury": 29.969259912138217
      },
      "surfacePressure": {
        "pascals": 101475.9140625,
        "hectopascals": 1014.759140625,
        "inchesOfMercury": 29.969259912138217
      },
      "windDirection": {
        "angle": {
          "radians": 2.5068660660370234,
          "degrees": 143.63284538848538,
          "percentage": -73.63790986581812,
          "degMinSec": {}
        },
        "radians": 2.5068660660370234,
        "degrees": 143.63284538848538
      },
      "windSpeed": {
        "metersPerSecond": 6.570514368371236,
        "knots": 12.772054931872216,
        "feetPerMinute": 1293.408381619625
      },
      "windXComp": {
        "metersPerSecond": 1.3576958213542298,
        "knots": 2.6391488761644912,
        "feetPerMinute": 267.26296551190865
      },
      "windHComp": {
        "metersPerSecond": 6.428710688905679,
        "knots": 12.49641070036517,
        "feetPerMinute": 1265.4942705953583
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9555677654668955,
            "degrees": 54.75000000000001,
            "percentage": 141.49672721156952,
            "degMinSec": {}
          },
          "radians": 0.9555677654668955,
          "degrees": 54.75000000000001,
          "degMinSec": {},
          "vrc": "N054.45.000.000",
          "nats": "544500N"
        },
        "lon": {
          "angle": {
            "radians": -0.10908254561742758,
            "degrees": -6.249969482421875,
            "percentage": -10.95172726625836,
            "degMinSec": {}
          },
          "radians": -0.10908254561742758,
          "degrees": -6.249969482421875,
          "degMinSec": {},
          "vrc": "E006.14.059.890",
          "nats": "0061500W"
        },
        "geoPotentialHeight": {
          "meters": 212.58175659179688,
          "feet": 697.4467302966309,
          "nauticalMiles": 0.11478496576230933,
          "statuteMiles": 0.13209217954135155
        },
        "levelPressure": {
          "pascals": 100000,
          "hectopascals": 1000,
          "inchesOfMercury": 29.533372711163615
        },
        "temp": {
          "kelvin": 285.9950866699219,
          "celsius": 12.845086669921898
        },
        "v": {
          "metersPerSecond": 5.29080057144165,
          "knots": 10.284490945993422,
          "feetPerMinute": 1041.4962088085174
        },
        "u": {
          "metersPerSecond": -3.8960349559783936,
          "knots": -7.573284172968864,
          "feetPerMinute": -766.9360394983291
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 101475.9140625,
          "hectopascals": 1014.759140625,
          "inchesOfMercury": 29.969259912138217
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9540598847689765,
            "degrees": 54.66360480000001,
            "percentage": 141.0450049177102,
            "degMinSec": {}
          },
          "radians": 0.9540598847689765,
          "degrees": 54.66360480000001,
          "degMinSec": {},
          "vrc": "N054.39.048.977",
          "nats": "543949N"
        },
        "lon": {
          "angle": {
            "radians": -0.10815966817564693,
            "degrees": -6.197092500000013,
            "percentage": -10.858342031858847,
            "degMinSec": {}
          },
          "radians": -0.10815966817564693,
          "degrees": -6.197092500000013,
          "degMinSec": {},
          "vrc": "E006.11.049.533",
          "nats": "0061150W"
        },
        "alt": {
          "meters": 81.68639738603528,
          "feet": 268,
          "nauticalMiles": 0.04410712601837758,
          "statuteMiles": 0.050757574133333386
        }
      },
      "onGround": true,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 17000,
      "departureAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGPH",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9765117164908275,
              "degrees": 55.95,
              "percentage": 147.97738036278378,
              "degMinSec": {}
            },
            "radians": 0.9765117164908275,
            "degrees": 55.95,
            "degMinSec": {},
            "vrc": "N055.57.000.000",
            "nats": "555700N"
          },
          "lon": {
            "angle": {
              "radians": -0.05886122902350799,
              "degrees": -3.3724999999999556,
              "percentage": -5.892930110347406,
              "degMinSec": {}
            },
            "radians": -0.05886122902350799,
            "degrees": -3.3724999999999556,
            "degMinSec": {},
            "vrc": "E003.22.021.000",
            "nats": "0032221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9540597477360087,
                  "degrees": 54.663596948589294,
                  "percentage": 141.04496395350284,
                  "degMinSec": {}
                },
                "radians": 0.9540597477360087,
                "degrees": 54.663596948589294,
                "degMinSec": {},
                "vrc": "N054.39.048.949",
                "nats": "543949N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10815957924996233,
                  "degrees": -6.197087404933596,
                  "percentage": -10.858333034443898,
                  "degMinSec": {}
                },
                "radians": -0.10815957924996233,
                "degrees": -6.197087404933596,
                "degMinSec": {},
                "vrc": "E006.11.049.515",
                "nats": "0061150W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "percentage": 142.1976644186302,
                  "degMinSec": {}
                },
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "degMinSec": {},
                "vrc": "N054.52.060.000",
                "nats": "545300N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "percentage": -9.028331842888736,
                  "degMinSec": {}
                },
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "degMinSec": {},
                "vrc": "E005.09.031.920",
                "nats": "0050932W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "BLACA"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 1.2117304947497765,
            "degrees": 69.42704325646135,
            "percentage": 266.42744595496794,
            "degMinSec": {}
          },
          "radians": 1.2117304947497765,
          "degrees": 69.42704325646135
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 1.2265327858114006,
            "degrees": 70.27515206141663,
            "percentage": 278.9079828055174,
            "degMinSec": {}
          },
          "radians": 1.2265327858114006,
          "degrees": 70.27515206141663
        },
        "legLength": {
          "meters": 70930.15643543784,
          "feet": 232710.49443964186,
          "nauticalMiles": 38.29922053749343,
          "statuteMiles": 44.07395586986861
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9540597477360087,
                  "degrees": 54.663596948589294,
                  "percentage": 141.04496395350284,
                  "degMinSec": {}
                },
                "radians": 0.9540597477360087,
                "degrees": 54.663596948589294,
                "degMinSec": {},
                "vrc": "N054.39.048.949",
                "nats": "543949N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10815957924996233,
                  "degrees": -6.197087404933596,
                  "percentage": -10.858333034443898,
                  "degMinSec": {}
                },
                "radians": -0.10815957924996233,
                "degrees": -6.197087404933596,
                "degMinSec": {},
                "vrc": "E006.11.049.515",
                "nats": "0061150W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "percentage": 142.1976644186302,
                  "degMinSec": {}
                },
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "degMinSec": {},
                "vrc": "N054.52.060.000",
                "nats": "545300N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "percentage": -9.028331842888736,
                  "degMinSec": {}
                },
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "degMinSec": {},
                "vrc": "E005.09.031.920",
                "nats": "0050932W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 0.46008763830309807,
              "degrees": 26.361079880909077,
              "percentage": 49.55579246914306,
              "degMinSec": {}
            },
            "radians": 0.46008763830309807,
            "degrees": 26.361079880909077
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.4638431622056389,
              "degrees": 26.576255550385167,
              "percentage": 50.02444654121546,
              "degMinSec": {}
            },
            "radians": 0.4638431622056389,
            "degrees": 26.576255550385167
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "GIRVA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BLACA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 37540.22363486868,
            "feet": 123163.46731022255,
            "nauticalMiles": 20.270099154896695,
            "statuteMiles": 23.326413516854494
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> BLACA; BLACA =(TF)=> GIRVA; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540597477360087,
                "degrees": 54.663596948589294,
                "percentage": 141.04496395350284,
                "degMinSec": {}
              },
              "radians": 0.9540597477360087,
              "degrees": 54.663596948589294,
              "degMinSec": {},
              "vrc": "N054.39.048.949",
              "nats": "543949N"
            },
            "lon": {
              "angle": {
                "radians": -0.10815957924996233,
                "degrees": -6.197087404933596,
                "percentage": -10.858333034443898,
                "degMinSec": {}
              },
              "radians": -0.10815957924996233,
              "degrees": -6.197087404933596,
              "degMinSec": {},
              "vrc": "E006.11.049.515",
              "nats": "0061150W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "percentage": 143.80375687141498,
                "degMinSec": {}
              },
              "radians": 0.96316959554411,
              "degrees": 55.18555278,
              "degMinSec": {},
              "vrc": "N055.11.007.990",
              "nats": "551108N"
            },
            "lon": {
              "angle": {
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "percentage": -8.566514145879498,
                "degMinSec": {}
              },
              "radians": -0.0854565076452456,
              "degrees": -4.896297220000025,
              "degMinSec": {},
              "vrc": "E004.53.046.670",
              "nats": "0045347W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 157,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 2.7401669256310974,
          "degrees": 157,
          "percentage": -42.447481620960474,
          "degMinSec": {}
        },
        "radians": 2.7401669256310974,
        "degrees": 157
      },
      "selectedAltitude": 5000,
      "selectedAltitudeLength": {
        "meters": 1523.9999512320016,
        "feet": 5000,
        "nauticalMiles": 0.8228941421339102,
        "statuteMiles": 0.9469696666666677
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "ON_GROUND"
  },
  {
    "callsign": "RRR01",
    "delayMs": 1620000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A400",
      "filedTas": 360,
      "origin": "EGVN",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 19000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "KISWO N864 WAL M146 IPSET P6 BELZU DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9436018379999008,
          "degrees": 54.064402858181545,
          "percentage": 137.96404206105063,
          "degMinSec": {}
        },
        "radians": 0.9436018379999008,
        "degrees": 54.064402858181545,
        "degMinSec": {},
        "vrc": "N054.03.051.850",
        "nats": "540352N"
      },
      "longitude": {
        "angle": {
          "radians": -0.08033979030234129,
          "degrees": -4.603130911290216,
          "percentage": -8.051308830991335,
          "degMinSec": {}
        },
        "radians": -0.08033979030234129,
        "degrees": -4.603130911290216,
        "degMinSec": {},
        "vrc": "E004.36.011.271",
        "nats": "0043611W"
      },
      "indicatedAltitude": {
        "meters": 3910.819972506674,
        "feet": 12830.774598598795,
        "nauticalMiles": 2.1116738512455044,
        "statuteMiles": 2.4300708689420496
      },
      "trueAltitude": {
        "meters": 4039.321747506674,
        "feet": 13252.368362089795,
        "nauticalMiles": 2.181059258912891,
        "statuteMiles": 2.5099181700784134
      },
      "pressureAltitude": {
        "meters": 3910.819972506674,
        "feet": 12830.774598598795,
        "nauticalMiles": 2.1116738512455044,
        "statuteMiles": 2.4300708689420496
      },
      "densityAltitude": {
        "meters": 4164.018906086584,
        "feet": 13661.479787845108,
        "nauticalMiles": 2.2483903380597106,
        "statuteMiles": 2.58740139217382
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.449634108168489,
          "degrees": 312.2410342885948,
          "percentage": -110.12603407403314,
          "degMinSec": {}
        },
        "radians": 5.449634108168489,
        "degrees": 312.2410342885948
      },
      "heading_True": {
        "angle": {
          "radians": 5.433789972984833,
          "degrees": 311.33323221253653,
          "percentage": -113.69454753286306,
          "degMinSec": {}
        },
        "radians": 5.433789972984833,
        "degrees": 311.33323221253653
      },
      "track_True": {
        "angle": {
          "radians": 5.464978258217576,
          "degrees": 313.1201893266229,
          "percentage": -106.7868825252123,
          "degMinSec": {}
        },
        "radians": 5.464978258217576,
        "degrees": 313.1201893266229
      },
      "track_Mag": {
        "angle": {
          "radians": 5.480822393401233,
          "degrees": 314.0279914026812,
          "percentage": -103.4518408506909,
          "degMinSec": {}
        },
        "radians": 5.480822393401233,
        "degrees": 314.0279914026812
      },
      "bank": {
        "radians": 0.002807805223155893,
        "degrees": 0.16087538898162096,
        "percentage": 0.28078126018761834,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.03421702163507619,
        "degrees": -1.9604909271976931,
        "percentage": -3.423038170702026,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 149.18844869697097,
        "knots": 289.9990708689148,
        "feetPerMinute": 29367.80580137821
      },
      "trueAirSpeed": {
        "metersPerSecond": 181.58105672129187,
        "knots": 352.9652476213429,
        "feetPerMinute": 35744.303648008994
      },
      "groundSpeed": {
        "metersPerSecond": 189.5336632635,
        "knots": 368.42387413277487,
        "feetPerMinute": 37309.77742688528
      },
      "machNumber": 0.5493036363229644,
      "verticalSpeed": {
        "metersPerSecond": -11.36016630516628,
        "knots": -22.08239111129964,
        "feetPerMinute": -2236.2532812385043
      },
      "flightPathAngle": {
        "radians": -0.05986583893373062,
        "degrees": -3.430059907912729,
        "percentage": -5.993745970800322,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -163.3659118479964,
        "knots": -317.55784755025667,
        "feetPerMinute": -32158.645093642826
      },
      "velocity_Y": {
        "metersPerSecond": -11.36016630516628,
        "knots": -22.08239111129964,
        "feetPerMinute": -2236.2532812385043
      },
      "velocity_Z": {
        "metersPerSecond": 96.09676558633217,
        "knots": 186.79712120439825,
        "feetPerMinute": 18916.686744375722
      },
      "heading_Velocity": {
        "radiansPerSecond": 0.0001468831254098608,
        "degreesPerSecond": 0.008415783167675803
      },
      "bank_Velocity": {
        "radiansPerSecond": 0.00015293193939434555,
        "degreesPerSecond": 0.008762354680046491
      },
      "pitch_Velocity": {
        "radiansPerSecond": -0.0006917759600215401,
        "degreesPerSecond": -0.03963584287784501
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.00572352626563033,
        "knotsPerSecond": 0.011125642190287922
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102730.3125,
        "hectopascals": 1027.303125,
        "inchesOfMercury": 30.339726077968105
      },
      "windDirection": {
        "angle": {
          "radians": 2.9111638288960915,
          "degrees": 166.79740086689097,
          "percentage": -23.459574199985955,
          "degMinSec": {}
        },
        "radians": 2.9111638288960915,
        "degrees": 166.79740086689097
      },
      "windSpeed": {
        "metersPerSecond": 9.764047036020655,
        "knots": 18.97978424668653,
        "feetPerMinute": 1922.0565646594803
      },
      "windXComp": {
        "metersPerSecond": 5.664805097463768,
        "knots": 11.01149739987436,
        "feetPerMinute": 1115.1191493577817
      },
      "windHComp": {
        "metersPerSecond": -7.952772958495184,
        "knots": -15.458949998733111,
        "feetPerMinute": -1565.5065379889604
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07853981633974527,
            "degrees": -4.500000000000026,
            "percentage": -7.8701706824618896,
            "degMinSec": {}
          },
          "radians": -0.07853981633974527,
          "degrees": -4.500000000000026,
          "degMinSec": {},
          "vrc": "E004.30.000.000",
          "nats": "0043000W"
        },
        "geoPotentialHeight": {
          "meters": 3770.177978515625,
          "feet": 12369.350719033204,
          "nauticalMiles": 2.0357332497384584,
          "statuteMiles": 2.3426799854571954
        },
        "levelPressure": {
          "pascals": 65000,
          "hectopascals": 650,
          "inchesOfMercury": 19.19669226225635
        },
        "temp": {
          "kelvin": 273.66998291015625,
          "celsius": 0.5199829101562727
        },
        "v": {
          "metersPerSecond": 9.505969047546387,
          "knots": 18.47812089725876,
          "feetPerMinute": 1871.2538093971252
        },
        "u": {
          "metersPerSecond": -2.230059862136841,
          "knots": -4.334888482655525,
          "feetPerMinute": -438.988175885582
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102730.3125,
          "hectopascals": 1027.303125,
          "inchesOfMercury": 30.339726077968105
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9436059022175229,
            "degrees": 54.06463572069831,
            "percentage": 137.96522207573702,
            "degMinSec": {}
          },
          "radians": 0.9436059022175229,
          "degrees": 54.06463572069831,
          "degMinSec": {},
          "vrc": "N054.03.052.689",
          "nats": "540353N"
        },
        "lon": {
          "angle": {
            "radians": -0.08034718529393636,
            "degrees": -4.603554613098148,
            "percentage": -8.052053124291843,
            "degMinSec": {}
          },
          "radians": -0.08034718529393636,
          "degrees": -4.603554613098148,
          "degMinSec": {},
          "vrc": "E004.36.012.797",
          "nats": "0043613W"
        },
        "alt": {
          "meters": 4037.0446507390357,
          "feet": 13244.897571930658,
          "nauticalMiles": 2.1798297250210776,
          "statuteMiles": 2.508503247745066
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0.00015293193939434555,
        "degreesPerSecond": 0.008762354680046491
      },
      "pitchRate": {
        "radiansPerSecond": -0.0006917759600215401,
        "degreesPerSecond": -0.03963584287784501
      },
      "yawRate": {
        "radiansPerSecond": 0.0001468831254098608,
        "degreesPerSecond": 0.008415783167675803
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 19000,
      "departureAirport": {
        "identifier": "EGVN",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9032076454808324,
              "degrees": 51.74998611,
              "percentage": 126.84933202368323,
              "degMinSec": {}
            },
            "radians": 0.9032076454808324,
            "degrees": 51.74998611,
            "degMinSec": {},
            "vrc": "N051.44.059.950",
            "nats": "514500N"
          },
          "lon": {
            "angle": {
              "radians": -0.02762764087429126,
              "degrees": -1.582947220000013,
              "percentage": -2.7634672289899354,
              "degMinSec": {}
            },
            "radians": -0.02762764087429126,
            "degrees": -1.582947220000013,
            "degMinSec": {},
            "vrc": "E001.34.058.610",
            "nats": "0013459W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "legType": "TRACK_TO_FIX",
        "initialTrueCourse": {
          "angle": {
            "radians": 5.460531278681644,
            "degrees": 312.8653961676329,
            "percentage": -107.74323687641969,
            "degMinSec": {}
          },
          "radians": 5.460531278681644,
          "degrees": 312.8653961676329
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.447293429813374,
            "degrees": 312.1069232976491,
            "percentage": -110.64531286515107,
            "degMinSec": {}
          },
          "radians": 5.447293429813374,
          "degrees": 312.1069232976491
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "IPSET"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPEG"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "legLength": {
          "meters": 82274.60605599961,
          "feet": 269929.8185327658,
          "nauticalMiles": 44.424733291576466,
          "statuteMiles": 51.123070055873455
        },
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "percentage": 140.28922751224925,
                  "degMinSec": {}
                },
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "degMinSec": {},
                "vrc": "N054.31.005.630",
                "nats": "543106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "percentage": -9.545200562109763,
                  "degMinSec": {}
                },
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "degMinSec": {},
                "vrc": "E005.27.008.920",
                "nats": "0052709W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.1474641895474593,
              "degrees": 180.3364142296336,
              "percentage": 0.5871603432202414,
              "degMinSec": {}
            },
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 6.165737882555982,
              "degrees": 353.27075825438664,
              "percentage": -11.799044018857247,
              "degMinSec": {}
            },
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16253794.83622146,
            "feet": 53326100.25046881,
            "nauticalMiles": 8776.347103791286,
            "statuteMiles": 10099.63987576395
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PEPEG =(TF)=> IPSET; IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 75263.27551969403,
      "crossTrackDistance_m": -30.145631372378368,
      "requiredTrueCourse": 312.80122169205947,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY691V",
    "delayMs": 1620000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A20N",
      "filedTas": 320,
      "origin": "EGAA",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 17000,
      "destination": "EGPH",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "BLACA P600 GIRVA"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9540598847689765,
          "degrees": 54.66360480000001,
          "percentage": 141.0450049177102,
          "degMinSec": {}
        },
        "radians": 0.9540598847689765,
        "degrees": 54.66360480000001,
        "degMinSec": {},
        "vrc": "N054.39.048.977",
        "nats": "543949N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10815966817564693,
          "degrees": -6.197092500000013,
          "percentage": -10.858342031858847,
          "degMinSec": {}
        },
        "radians": -0.10815966817564693,
        "degrees": -6.197092500000013,
        "degMinSec": {},
        "vrc": "E006.11.049.533",
        "nats": "0061150W"
      },
      "indicatedAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "trueAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "pressureAltitude": {
        "meters": 67.88681551103528,
        "feet": 222.72577980122497,
        "nauticalMiles": 0.036655947900127044,
        "statuteMiles": 0.04218291149128793
      },
      "densityAltitude": {
        "meters": -72.58308962302895,
        "feet": -238.1335037588183,
        "nauticalMiles": -0.039191733057791014,
        "statuteMiles": -0.045101040935330766
      },
      "heading_Mag": {
        "angle": {
          "radians": 2.742757648739527,
          "degrees": 157.14843750000003,
          "percentage": -42.1420651362081,
          "degMinSec": {}
        },
        "radians": 2.742757648739527,
        "degrees": 157.14843750000003
      },
      "heading_True": {
        "angle": {
          "radians": 2.7150001683767773,
          "degrees": 155.5580510252972,
          "percentage": -45.450321426367985,
          "degMinSec": {}
        },
        "radians": 2.7150001683767773,
        "degrees": 155.5580510252972
      },
      "track_True": {
        "angle": {
          "radians": 2.7150001683767773,
          "degrees": 155.5580510252972,
          "percentage": -45.450321426367985,
          "degMinSec": {}
        },
        "radians": 2.7150001683767773,
        "degrees": 155.5580510252972
      },
      "track_Mag": {
        "angle": {
          "radians": 2.742757648739527,
          "degrees": 157.14843750000003,
          "percentage": -42.1420651362081,
          "degMinSec": {}
        },
        "radians": 2.742757648739527,
        "degrees": 157.14843750000003
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 6.451128390606732,
        "knots": 12.539987215310552,
        "feetPerMinute": 1269.9072041422914
      },
      "trueAirSpeed": {
        "metersPerSecond": 6.428710688905679,
        "knots": 12.49641070036517,
        "feetPerMinute": 1265.4942705953583
      },
      "groundSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "machNumber": 0.018934363403326642,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -0,
        "knots": -0,
        "feetPerMinute": -0
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101475.9140625,
        "hectopascals": 1014.759140625,
        "inchesOfMercury": 29.969259912138217
      },
      "surfacePressure": {
        "pascals": 101475.9140625,
        "hectopascals": 1014.759140625,
        "inchesOfMercury": 29.969259912138217
      },
      "windDirection": {
        "angle": {
          "radians": 2.5068660660370234,
          "degrees": 143.63284538848538,
          "percentage": -73.63790986581812,
          "degMinSec": {}
        },
        "radians": 2.5068660660370234,
        "degrees": 143.63284538848538
      },
      "windSpeed": {
        "metersPerSecond": 6.570514368371236,
        "knots": 12.772054931872216,
        "feetPerMinute": 1293.408381619625
      },
      "windXComp": {
        "metersPerSecond": 1.3576958213542298,
        "knots": 2.6391488761644912,
        "feetPerMinute": 267.26296551190865
      },
      "windHComp": {
        "metersPerSecond": 6.428710688905679,
        "knots": 12.49641070036517,
        "feetPerMinute": 1265.4942705953583
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9555677654668955,
            "degrees": 54.75000000000001,
            "percentage": 141.49672721156952,
            "degMinSec": {}
          },
          "radians": 0.9555677654668955,
          "degrees": 54.75000000000001,
          "degMinSec": {},
          "vrc": "N054.45.000.000",
          "nats": "544500N"
        },
        "lon": {
          "angle": {
            "radians": -0.10908254561742758,
            "degrees": -6.249969482421875,
            "percentage": -10.95172726625836,
            "degMinSec": {}
          },
          "radians": -0.10908254561742758,
          "degrees": -6.249969482421875,
          "degMinSec": {},
          "vrc": "E006.14.059.890",
          "nats": "0061500W"
        },
        "geoPotentialHeight": {
          "meters": 212.58175659179688,
          "feet": 697.4467302966309,
          "nauticalMiles": 0.11478496576230933,
          "statuteMiles": 0.13209217954135155
        },
        "levelPressure": {
          "pascals": 100000,
          "hectopascals": 1000,
          "inchesOfMercury": 29.533372711163615
        },
        "temp": {
          "kelvin": 285.9950866699219,
          "celsius": 12.845086669921898
        },
        "v": {
          "metersPerSecond": 5.29080057144165,
          "knots": 10.284490945993422,
          "feetPerMinute": 1041.4962088085174
        },
        "u": {
          "metersPerSecond": -3.8960349559783936,
          "knots": -7.573284172968864,
          "feetPerMinute": -766.9360394983291
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 101475.9140625,
          "hectopascals": 1014.759140625,
          "inchesOfMercury": 29.969259912138217
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9540598847689765,
            "degrees": 54.66360480000001,
            "percentage": 141.0450049177102,
            "degMinSec": {}
          },
          "radians": 0.9540598847689765,
          "degrees": 54.66360480000001,
          "degMinSec": {},
          "vrc": "N054.39.048.977",
          "nats": "543949N"
        },
        "lon": {
          "angle": {
            "radians": -0.10815966817564693,
            "degrees": -6.197092500000013,
            "percentage": -10.858342031858847,
            "degMinSec": {}
          },
          "radians": -0.10815966817564693,
          "degrees": -6.197092500000013,
          "degMinSec": {},
          "vrc": "E006.11.049.533",
          "nats": "0061150W"
        },
        "alt": {
          "meters": 81.68639738603528,
          "feet": 268,
          "nauticalMiles": 0.04410712601837758,
          "statuteMiles": 0.050757574133333386
        }
      },
      "onGround": true,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 17000,
      "departureAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGPH",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9765117164908275,
              "degrees": 55.95,
              "percentage": 147.97738036278378,
              "degMinSec": {}
            },
            "radians": 0.9765117164908275,
            "degrees": 55.95,
            "degMinSec": {},
            "vrc": "N055.57.000.000",
            "nats": "555700N"
          },
          "lon": {
            "angle": {
              "radians": -0.05886122902350799,
              "degrees": -3.3724999999999556,
              "percentage": -5.892930110347406,
              "degMinSec": {}
            },
            "radians": -0.05886122902350799,
            "degrees": -3.3724999999999556,
            "degMinSec": {},
            "vrc": "E003.22.021.000",
            "nats": "0032221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9540597477360087,
                  "degrees": 54.663596948589294,
                  "percentage": 141.04496395350284,
                  "degMinSec": {}
                },
                "radians": 0.9540597477360087,
                "degrees": 54.663596948589294,
                "degMinSec": {},
                "vrc": "N054.39.048.949",
                "nats": "543949N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10815957924996233,
                  "degrees": -6.197087404933596,
                  "percentage": -10.858333034443898,
                  "degMinSec": {}
                },
                "radians": -0.10815957924996233,
                "degrees": -6.197087404933596,
                "degMinSec": {},
                "vrc": "E006.11.049.515",
                "nats": "0061150W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "percentage": 142.1976644186302,
                  "degMinSec": {}
                },
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "degMinSec": {},
                "vrc": "N054.52.060.000",
                "nats": "545300N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "percentage": -9.028331842888736,
                  "degMinSec": {}
                },
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "degMinSec": {},
                "vrc": "E005.09.031.920",
                "nats": "0050932W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "BLACA"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 1.2117304947497765,
            "degrees": 69.42704325646135,
            "percentage": 266.42744595496794,
            "degMinSec": {}
          },
          "radians": 1.2117304947497765,
          "degrees": 69.42704325646135
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 1.2265327858114006,
            "degrees": 70.27515206141663,
            "percentage": 278.9079828055174,
            "degMinSec": {}
          },
          "radians": 1.2265327858114006,
          "degrees": 70.27515206141663
        },
        "legLength": {
          "meters": 70930.15643543784,
          "feet": 232710.49443964186,
          "nauticalMiles": 38.29922053749343,
          "statuteMiles": 44.07395586986861
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9540597477360087,
                  "degrees": 54.663596948589294,
                  "percentage": 141.04496395350284,
                  "degMinSec": {}
                },
                "radians": 0.9540597477360087,
                "degrees": 54.663596948589294,
                "degMinSec": {},
                "vrc": "N054.39.048.949",
                "nats": "543949N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10815957924996233,
                  "degrees": -6.197087404933596,
                  "percentage": -10.858333034443898,
                  "degMinSec": {}
                },
                "radians": -0.10815957924996233,
                "degrees": -6.197087404933596,
                "degMinSec": {},
                "vrc": "E006.11.049.515",
                "nats": "0061150W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "percentage": 142.1976644186302,
                  "degMinSec": {}
                },
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "degMinSec": {},
                "vrc": "N054.52.060.000",
                "nats": "545300N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "percentage": -9.028331842888736,
                  "degMinSec": {}
                },
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "degMinSec": {},
                "vrc": "E005.09.031.920",
                "nats": "0050932W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 0.46008763830309807,
              "degrees": 26.361079880909077,
              "percentage": 49.55579246914306,
              "degMinSec": {}
            },
            "radians": 0.46008763830309807,
            "degrees": 26.361079880909077
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.4638431622056389,
              "degrees": 26.576255550385167,
              "percentage": 50.02444654121546,
              "degMinSec": {}
            },
            "radians": 0.4638431622056389,
            "degrees": 26.576255550385167
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "GIRVA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BLACA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 37540.22363486868,
            "feet": 123163.46731022255,
            "nauticalMiles": 20.270099154896695,
            "statuteMiles": 23.326413516854494
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> BLACA; BLACA =(TF)=> GIRVA; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540597477360087,
                "degrees": 54.663596948589294,
                "percentage": 141.04496395350284,
                "degMinSec": {}
              },
              "radians": 0.9540597477360087,
              "degrees": 54.663596948589294,
              "degMinSec": {},
              "vrc": "N054.39.048.949",
              "nats": "543949N"
            },
            "lon": {
              "angle": {
                "radians": -0.10815957924996233,
                "degrees": -6.197087404933596,
                "percentage": -10.858333034443898,
                "degMinSec": {}
              },
              "radians": -0.10815957924996233,
              "degrees": -6.197087404933596,
              "degMinSec": {},
              "vrc": "E006.11.049.515",
              "nats": "0061150W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "percentage": 143.80375687141498,
                "degMinSec": {}
              },
              "radians": 0.96316959554411,
              "degrees": 55.18555278,
              "degMinSec": {},
              "vrc": "N055.11.007.990",
              "nats": "551108N"
            },
            "lon": {
              "angle": {
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "percentage": -8.566514145879498,
                "degMinSec": {}
              },
              "radians": -0.0854565076452456,
              "degrees": -4.896297220000025,
              "degMinSec": {},
              "vrc": "E004.53.046.670",
              "nats": "0045347W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 157,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 2.7401669256310974,
          "degrees": 157,
          "percentage": -42.447481620960474,
          "degMinSec": {}
        },
        "radians": 2.7401669256310974,
        "degrees": 157
      },
      "selectedAltitude": 5000,
      "selectedAltitudeLength": {
        "meters": 1523.9999512320016,
        "feet": 5000,
        "nauticalMiles": 0.8228941421339102,
        "statuteMiles": 0.9469696666666677
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "ON_GROUND"
  },
  {
    "callsign": "EZY616D",
    "delayMs": 1680000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A320",
      "filedTas": 400,
      "origin": "EGKK",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 32000,
      "destination": "EGAC",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "LAM L10 BPK UN601 LESTA UP6 TUPEM DCT REMSI DCT NINEB M148 MAGEE DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.946098727757807,
          "degrees": 54.20746410321901,
          "percentage": 138.691498414227,
          "degMinSec": {}
        },
        "radians": 0.946098727757807,
        "degrees": 54.20746410321901,
        "degMinSec": {},
        "vrc": "N054.12.026.871",
        "nats": "541227N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07773993351717756,
          "degrees": -4.454170090161884,
          "percentage": -7.78969200523324,
          "degMinSec": {}
        },
        "radians": -0.07773993351717756,
        "degrees": -4.454170090161884,
        "degMinSec": {},
        "vrc": "E004.27.015.012",
        "nats": "0042715W"
      },
      "indicatedAltitude": {
        "meters": 5377.191202097333,
        "feet": 17641.703983489013,
        "nauticalMiles": 2.9034509730547153,
        "statuteMiles": 3.341231708135323
      },
      "trueAltitude": {
        "meters": 5391.283677722333,
        "feet": 17687.93914121854,
        "nauticalMiles": 2.911060301145968,
        "statuteMiles": 3.3499883665160044
      },
      "pressureAltitude": {
        "meters": 5377.191202097333,
        "feet": 17641.703983489013,
        "nauticalMiles": 2.9034509730547153,
        "statuteMiles": 3.341231708135323
      },
      "densityAltitude": {
        "meters": 5441.097585279885,
        "feet": 17851.370601689658,
        "nauticalMiles": 2.9379576594383825,
        "statuteMiles": 3.3809412936450407
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.426783609933253,
          "degrees": 310.93179717994457,
          "percentage": -115.31378295106383,
          "degMinSec": {}
        },
        "radians": 5.426783609933253,
        "degrees": 310.93179717994457
      },
      "heading_True": {
        "angle": {
          "radians": 5.411791908002677,
          "degrees": 310.07283593160446,
          "percentage": -118.86815245303721,
          "degMinSec": {}
        },
        "radians": 5.411791908002677,
        "degrees": 310.07283593160446
      },
      "track_True": {
        "angle": {
          "radians": 5.441114285424257,
          "degrees": 311.75288440315074,
          "percentage": -112.02918099336678,
          "degMinSec": {}
        },
        "radians": 5.441114285424257,
        "degrees": 311.75288440315074
      },
      "track_Mag": {
        "angle": {
          "radians": 5.456105987354835,
          "degrees": 312.61184565149097,
          "percentage": -108.70406807262198,
          "degMinSec": {}
        },
        "radians": 5.456105987354835,
        "degrees": 312.61184565149097
      },
      "bank": {
        "radians": 0.000884052842366203,
        "degrees": 0.050652496734127694,
        "percentage": 0.08840530726766062,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.034900893252734544,
        "degrees": -1.9996738846183009,
        "percentage": -3.491507076482074,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 149.1891819560275,
        "knots": 290.0004962101323,
        "feetPerMinute": 29367.9501437168
      },
      "trueAirSpeed": {
        "metersPerSecond": 193.25961741124857,
        "knots": 375.66654774715107,
        "feetPerMinute": 38043.23299125124
      },
      "groundSpeed": {
        "metersPerSecond": 197.09772326688892,
        "knots": 383.1272267860024,
        "feetPerMinute": 38798.76566417639
      },
      "machNumber": 0.5950498606581323,
      "verticalSpeed": {
        "metersPerSecond": -11.555930558807356,
        "knots": -22.462926281154324,
        "feetPerMinute": -2274.7895528734516
      },
      "flightPathAngle": {
        "radians": -0.05856341880018735,
        "degrees": -3.355436731107835,
        "percentage": -5.863046192146794,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -132.2021516633839,
        "knots": -256.9803592979588,
        "feetPerMinute": -26024.046435797783
      },
      "velocity_Y": {
        "metersPerSecond": -11.555930558807356,
        "knots": -22.462926281154324,
        "feetPerMinute": -2274.7895528734516
      },
      "velocity_Z": {
        "metersPerSecond": -146.18516892134704,
        "knots": -284.1611634967469,
        "feetPerMinute": -28776.608976234733
      },
      "heading_Velocity": {
        "radiansPerSecond": 4.398869963558323E-05,
        "degreesPerSecond": 0.0025203668353875816
      },
      "bank_Velocity": {
        "radiansPerSecond": 0.0015252992920978415,
        "degreesPerSecond": 0.08739321193149847
      },
      "pitch_Velocity": {
        "radiansPerSecond": -1.897262384016607E-05,
        "degreesPerSecond": -0.0010870512723308043
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": -9.010582493718111E-05,
        "knotsPerSecond": -0.00017515166716918987
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 101479.1171875,
        "hectopascals": 1014.791171875,
        "inchesOfMercury": 29.97020590298287
      },
      "windDirection": {
        "angle": {
          "radians": 3.245805339143898,
          "degrees": 185.9709470539742,
          "percentage": 10.459159143069828,
          "degMinSec": {}
        },
        "radians": 3.245805339143898,
        "degrees": 185.9709470539742
      },
      "windSpeed": {
        "metersPerSecond": 6.84561548165803,
        "knots": 13.306808580328072,
        "feetPerMinute": 1347.5621458105759
      },
      "windXComp": {
        "metersPerSecond": 5.668456118169688,
        "knots": 11.018594414567438,
        "feetPerMinute": 1115.8378542441503
      },
      "windHComp": {
        "metersPerSecond": -3.838105855640349,
        "knots": -7.460679038851358,
        "feetPerMinute": -755.5326729251449
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9468411192069238,
            "degrees": 54.25,
            "percentage": 138.908762759763,
            "degMinSec": {}
          },
          "radians": 0.9468411192069238,
          "degrees": 54.25,
          "degMinSec": {},
          "vrc": "N054.15.000.000",
          "nats": "541500N"
        },
        "lon": {
          "angle": {
            "radians": -0.07853981633974527,
            "degrees": -4.500000000000026,
            "percentage": -7.8701706824618896,
            "degMinSec": {}
          },
          "radians": -0.07853981633974527,
          "degrees": -4.500000000000026,
          "degMinSec": {},
          "vrc": "E004.30.000.000",
          "nats": "0043000W"
        },
        "geoPotentialHeight": {
          "meters": 5087.11572265625,
          "feet": 16690.01274751953,
          "nauticalMiles": 2.7468227444148217,
          "statuteMiles": 3.160987161636201
        },
        "levelPressure": {
          "pascals": 55000,
          "hectopascals": 550,
          "inchesOfMercury": 16.243354991139988
        },
        "temp": {
          "kelvin": 264.46099853515625,
          "celsius": -8.689001464843727
        },
        "v": {
          "metersPerSecond": 6.808476448059082,
          "knots": 13.234616092700957,
          "feetPerMinute": 1340.2513121910094
        },
        "u": {
          "metersPerSecond": 0.712109386920929,
          "knots": 1.384229559109926,
          "feetPerMinute": 140.17901765913965
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 101479.1171875,
          "hectopascals": 1014.791171875,
          "inchesOfMercury": 29.97020590298287
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.946098727757807,
            "degrees": 54.20746410321901,
            "percentage": 138.691498414227,
            "degMinSec": {}
          },
          "radians": 0.946098727757807,
          "degrees": 54.20746410321901,
          "degMinSec": {},
          "vrc": "N054.12.026.871",
          "nats": "541227N"
        },
        "lon": {
          "angle": {
            "radians": -0.07773993351717756,
            "degrees": -4.454170090161884,
            "percentage": -7.78969200523324,
            "degMinSec": {}
          },
          "radians": -0.07773993351717756,
          "degrees": -4.454170090161884,
          "degMinSec": {},
          "vrc": "E004.27.015.012",
          "nats": "0042715W"
        },
        "alt": {
          "meters": 5391.283677722333,
          "feet": 17687.93914121854,
          "nauticalMiles": 2.911060301145968,
          "statuteMiles": 3.3499883665160044
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0.0015252992920978415,
        "degreesPerSecond": 0.08739321193149847
      },
      "pitchRate": {
        "radiansPerSecond": -1.897262384016607E-05,
        "degreesPerSecond": -0.0010870512723308043
      },
      "yawRate": {
        "radiansPerSecond": 4.398869963558323E-05,
        "degreesPerSecond": 0.0025203668353875816
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 32000,
      "departureAirport": {
        "identifier": "EGKK",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.8927019755149921,
              "degrees": 51.14805556,
              "percentage": 124.14427272614329,
              "degMinSec": {}
            },
            "radians": 0.8927019755149921,
            "degrees": 51.14805556,
            "degMinSec": {},
            "vrc": "N051.08.053.000",
            "nats": "510853N"
          },
          "lon": {
            "angle": {
              "radians": -0.003320973754386003,
              "degrees": -0.19027778000003365,
              "percentage": -0.33209859632987887,
              "degMinSec": {}
            },
            "radians": -0.003320973754386003,
            "degrees": -0.19027778000003365,
            "degMinSec": {},
            "vrc": "E000.11.025.000",
            "nats": "0001125E"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAC",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9532649005591953,
              "degrees": 54.61805556,
              "percentage": 140.80762090061415,
              "degMinSec": {}
            },
            "radians": 0.9532649005591953,
            "degrees": 54.61805556,
            "degMinSec": {},
            "vrc": "N054.37.005.000",
            "nats": "543705N"
          },
          "lon": {
            "angle": {
              "radians": -0.10249446032336706,
              "degrees": -5.872500000000004,
              "percentage": -10.285488024374672,
              "degMinSec": {}
            },
            "radians": -0.10249446032336706,
            "degrees": -5.872500000000004,
            "degMinSec": {},
            "vrc": "E005.52.021.000",
            "nats": "0055221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "legType": "TRACK_TO_FIX",
        "initialTrueCourse": {
          "angle": {
            "radians": 5.438593895987405,
            "degrees": 311.60847672568974,
            "percentage": -112.5991529192575,
            "degMinSec": {}
          },
          "radians": 5.438593895987405,
          "degrees": 311.60847672568974
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.420779718567154,
            "degrees": 310.5877995440121,
            "percentage": -116.72229413505232,
            "degMinSec": {}
          },
          "radians": 5.420779718567154,
          "degrees": 310.5877995440121
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "percentage": 141.71458787308552,
                  "degMinSec": {}
                },
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "degMinSec": {},
                "vrc": "N054.47.029.530",
                "nats": "544730N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "percentage": -9.819779969455576,
                  "degMinSec": {}
                },
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "degMinSec": {},
                "vrc": "E005.36.030.020",
                "nats": "0053630W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MAGEE"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "percentage": 138.4284080202272,
                  "degMinSec": {}
                },
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "degMinSec": {},
                "vrc": "N054.09.021.020",
                "nats": "540921N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "percentage": -7.6142444922198695,
                  "degMinSec": {}
                },
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "degMinSec": {},
                "vrc": "E004.21.015.260",
                "nats": "0042115W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MASOP"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "legLength": {
          "meters": 107526.38245727522,
          "feet": 352776.8566211268,
          "nauticalMiles": 58.05960175878791,
          "statuteMiles": 66.81379646444465
        },
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "percentage": 138.4284080202272,
                  "degMinSec": {}
                },
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "degMinSec": {},
                "vrc": "N054.09.021.020",
                "nats": "540921N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "percentage": -7.6142444922198695,
                  "degMinSec": {}
                },
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "degMinSec": {},
                "vrc": "E004.21.015.260",
                "nats": "0042115W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "percentage": 141.71458787308552,
                  "degMinSec": {}
                },
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "degMinSec": {},
                "vrc": "N054.47.029.530",
                "nats": "544730N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "percentage": -9.819779969455576,
                  "degMinSec": {}
                },
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "degMinSec": {},
                "vrc": "E005.36.030.020",
                "nats": "0053630W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [],
      "asString": "MASOP =(TF)=> MAGEE; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "percentage": 138.4284080202272,
                "degMinSec": {}
              },
              "radians": 0.9451976978100912,
              "degrees": 54.15583889,
              "degMinSec": {},
              "vrc": "N054.09.021.020",
              "nats": "540921N"
            },
            "lon": {
              "angle": {
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "percentage": -7.6142444922198695,
                "degMinSec": {}
              },
              "radians": -0.07599580504888337,
              "degrees": -4.35423889000001,
              "degMinSec": {},
              "vrc": "E004.21.015.260",
              "nats": "0042115W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 99443.58616106295,
      "crossTrackDistance_m": -22.29096890572554,
      "requiredTrueCourse": 311.53233940912,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY613G",
    "delayMs": 1800000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A320",
      "filedTas": 400,
      "origin": "EGPH",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 24000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "GOSAM P600 BLACA DCT BELZU"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9631424658619565,
          "degrees": 55.18399836371308,
          "percentage": 143.7954339397244,
          "degMinSec": {}
        },
        "radians": 0.9631424658619565,
        "degrees": 55.18399836371308,
        "degMinSec": {},
        "vrc": "N055.11.002.394",
        "nats": "551102N"
      },
      "longitude": {
        "angle": {
          "radians": -0.0854823663307549,
          "degrees": -4.897778813543464,
          "percentage": -8.569118996640173,
          "degMinSec": {}
        },
        "radians": -0.0854823663307549,
        "degrees": -4.897778813543464,
        "degMinSec": {},
        "vrc": "E004.53.052.004",
        "nats": "0045352W"
      },
      "indicatedAltitude": {
        "meters": 5854.64602531067,
        "feet": 19208.15686568026,
        "nauticalMiles": 3.1612559531915063,
        "statuteMiles": 3.637908380874859
      },
      "trueAltitude": {
        "meters": 5956.66663343567,
        "feet": 19542.870157641082,
        "nauticalMiles": 3.2163426746412904,
        "statuteMiles": 3.7013010477782684
      },
      "pressureAltitude": {
        "meters": 5854.64602531067,
        "feet": 19208.15686568026,
        "nauticalMiles": 3.1612559531915063,
        "statuteMiles": 3.637908380874859
      },
      "densityAltitude": {
        "meters": 5954.985017563202,
        "feet": 19537.353045022053,
        "nauticalMiles": 3.2154346747101523,
        "statuteMiles": 3.7002561401187077
      },
      "heading_Mag": {
        "angle": {
          "radians": 3.6179411804029424,
          "degrees": 207.2927601636678,
          "percentage": 51.59784771173474,
          "degMinSec": {}
        },
        "radians": 3.6179411804029424,
        "degrees": 207.2927601636678
      },
      "heading_True": {
        "angle": {
          "radians": 3.5978806381103396,
          "degrees": 206.14337575555794,
          "percentage": 49.08340329481266,
          "degMinSec": {}
        },
        "radians": 3.5978806381103396,
        "degrees": 206.14337575555794
      },
      "track_True": {
        "angle": {
          "radians": 3.6030972126588185,
          "degrees": 206.44226346070116,
          "percentage": 49.73240508350329,
          "degMinSec": {}
        },
        "radians": 3.6030972126588185,
        "degrees": 206.44226346070116
      },
      "track_Mag": {
        "angle": {
          "radians": 3.6231577549514213,
          "degrees": 207.59164786881098,
          "percentage": 52.26017676669243,
          "degMinSec": {}
        },
        "radians": 3.6231577549514213,
        "degrees": 207.59164786881098
      },
      "bank": {
        "radians": 0.010730252400148717,
        "degrees": 0.6147981756386433,
        "percentage": 1.0730664240182066,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.03631063111280581,
        "degrees": -2.0804459142201885,
        "percentage": -3.632659759495232,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 148.75040059240658,
        "knots": 289.147573689146,
        "feetPerMinute": 29281.57585677547
      },
      "trueAirSpeed": {
        "metersPerSecond": 197.6335857635072,
        "knots": 384.1688598848789,
        "feetPerMinute": 38904.2504109807
      },
      "groundSpeed": {
        "metersPerSecond": 186.54558089313227,
        "knots": 362.6155081456298,
        "feetPerMinute": 36721.57221704544
      },
      "machNumber": 0.6142353590425087,
      "verticalSpeed": {
        "metersPerSecond": -11.899983778739722,
        "knots": -23.131712068400535,
        "feetPerMinute": -2342.516566838426
      },
      "flightPathAngle": {
        "radians": -0.06370497424380168,
        "degrees": -3.6500261581594495,
        "percentage": -6.379129283988213,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -146.45625514637885,
        "knots": -284.68811282875765,
        "feetPerMinute": -28829.972408066737
      },
      "velocity_Y": {
        "metersPerSecond": -11.899983778739722,
        "knots": -23.131712068400535,
        "feetPerMinute": -2342.516566838426
      },
      "velocity_Z": {
        "metersPerSecond": 115.54141715962692,
        "knots": 224.5944904972378,
        "feetPerMinute": 22744.37418443942
      },
      "heading_Velocity": {
        "radiansPerSecond": 0.0005641349364975886,
        "degreesPerSecond": 0.03232255093719253
      },
      "bank_Velocity": {
        "radiansPerSecond": 0.006239722579037696,
        "degreesPerSecond": 0.3575097691113452
      },
      "pitch_Velocity": {
        "radiansPerSecond": -0.0008047075107768947,
        "degreesPerSecond": -0.04610634410999428
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.0391669174606718,
        "knotsPerSecond": 0.07613437750442212
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102440.7109375,
        "hectopascals": 1024.407109375,
        "inchesOfMercury": 30.254196969137624
      },
      "windDirection": {
        "angle": {
          "radians": 3.5051656834368377,
          "degrees": 200.83120015501956,
          "percentage": 38.04876161430419,
          "degMinSec": {}
        },
        "radians": 3.5051656834368377,
        "degrees": 200.83120015501956
      },
      "windSpeed": {
        "metersPerSecond": 11.135832753639285,
        "knots": 21.6463216831652,
        "feetPerMinute": 2192.093131886995
      },
      "windXComp": {
        "metersPerSecond": 1.0309796853318924,
        "knots": 2.004063675454287,
        "feetPerMinute": 202.94876344945715
      },
      "windHComp": {
        "metersPerSecond": 11.088004870374943,
        "knots": 21.55335173924911,
        "feetPerMinute": 2182.678193935256
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.964294411726867,
            "degrees": 55.25,
            "percentage": 144.1494043228611,
            "degMinSec": {}
          },
          "radians": 0.964294411726867,
          "degrees": 55.25,
          "degMinSec": {},
          "vrc": "N055.15.000.000",
          "nats": "551500N"
        },
        "lon": {
          "angle": {
            "radians": -0.08726646259971638,
            "degrees": -4.999999999999995,
            "percentage": -8.74886635259239,
            "degMinSec": {}
          },
          "radians": -0.08726646259971638,
          "degrees": -4.999999999999995,
          "degMinSec": {},
          "vrc": "E004.59.060.000",
          "nats": "0056000W"
        },
        "geoPotentialHeight": {
          "meters": 5808.93115234375,
          "feet": 19058.173681855467,
          "nauticalMiles": 3.136571896513904,
          "statuteMiles": 3.6095024757564262
        },
        "levelPressure": {
          "pascals": 50000,
          "hectopascals": 500,
          "inchesOfMercury": 14.766686355581808
        },
        "temp": {
          "kelvin": 258.5806884765625,
          "celsius": -14.569311523437477
        },
        "v": {
          "metersPerSecond": 10.407907485961914,
          "knots": 20.23134851914215,
          "feetPerMinute": 2048.8007517745973
        },
        "u": {
          "metersPerSecond": 3.9600799083709717,
          "knots": 7.697777569407463,
          "feetPerMinute": 779.5433139947892
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102440.7109375,
          "hectopascals": 1024.407109375,
          "inchesOfMercury": 30.254196969137624
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9631424658619565,
            "degrees": 55.18399836371308,
            "percentage": 143.7954339397244,
            "degMinSec": {}
          },
          "radians": 0.9631424658619565,
          "degrees": 55.18399836371308,
          "degMinSec": {},
          "vrc": "N055.11.002.394",
          "nats": "551102N"
        },
        "lon": {
          "angle": {
            "radians": -0.0854823663307549,
            "degrees": -4.897778813543464,
            "percentage": -8.569118996640173,
            "degMinSec": {}
          },
          "radians": -0.0854823663307549,
          "degrees": -4.897778813543464,
          "degMinSec": {},
          "vrc": "E004.53.052.004",
          "nats": "0045352W"
        },
        "alt": {
          "meters": 5956.66663343567,
          "feet": 19542.870157641082,
          "nauticalMiles": 3.2163426746412904,
          "statuteMiles": 3.7013010477782684
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0.006239722579037696,
        "degreesPerSecond": 0.3575097691113452
      },
      "pitchRate": {
        "radiansPerSecond": -0.0008047075107768947,
        "degreesPerSecond": -0.04610634410999428
      },
      "yawRate": {
        "radiansPerSecond": 0.0005641349364975886,
        "degreesPerSecond": 0.03232255093719253
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 24000,
      "departureAirport": {
        "identifier": "EGPH",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9765117164908275,
              "degrees": 55.95,
              "percentage": 147.97738036278378,
              "degMinSec": {}
            },
            "radians": 0.9765117164908275,
            "degrees": 55.95,
            "degMinSec": {},
            "vrc": "N055.57.000.000",
            "nats": "555700N"
          },
          "lon": {
            "angle": {
              "radians": -0.05886122902350799,
              "degrees": -3.3724999999999556,
              "percentage": -5.892930110347406,
              "degMinSec": {}
            },
            "radians": -0.05886122902350799,
            "degrees": -3.3724999999999556,
            "degMinSec": {},
            "vrc": "E003.22.021.000",
            "nats": "0032221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9654025891289202,
                  "degrees": 55.31349388808942,
                  "percentage": 144.49103668203495,
                  "degMinSec": {}
                },
                "radians": 0.9654025891289202,
                "degrees": 55.31349388808942,
                "degMinSec": {},
                "vrc": "N055.18.048.578",
                "nats": "551849N"
              },
              "lon": {
                "angle": {
                  "radians": -0.08349884771887339,
                  "degrees": -4.784131568497007,
                  "percentage": -8.369344335413745,
                  "degMinSec": {}
                },
                "radians": -0.08349884771887339,
                "degrees": -4.784131568497007,
                "degMinSec": {},
                "vrc": "E004.47.002.874",
                "nats": "0044703W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "percentage": 143.80375687141498,
                  "degMinSec": {}
                },
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "degMinSec": {},
                "vrc": "N055.11.007.990",
                "nats": "551108N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "percentage": -8.566514145879498,
                  "degMinSec": {}
                },
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "degMinSec": {},
                "vrc": "E004.53.046.670",
                "nats": "0045347W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "GIRVA"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 3.605821287363006,
            "degrees": 206.59834144432944,
            "percentage": 50.072649210221456,
            "degMinSec": {}
          },
          "radians": 3.605821287363006,
          "degrees": 206.59834144432944
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 3.604212790192994,
            "degrees": 206.50618134512902,
            "percentage": 49.87163185249076,
            "degMinSec": {}
          },
          "radians": 3.604212790192994,
          "degrees": 206.50618134512902
        },
        "legLength": {
          "meters": 15903.819663842667,
          "feet": 52177.887705921574,
          "nauticalMiles": 8.587375628424766,
          "statuteMiles": 9.882175385649473
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9654025891289202,
                  "degrees": 55.31349388808942,
                  "percentage": 144.49103668203495,
                  "degMinSec": {}
                },
                "radians": 0.9654025891289202,
                "degrees": 55.31349388808942,
                "degMinSec": {},
                "vrc": "N055.18.048.578",
                "nats": "551849N"
              },
              "lon": {
                "angle": {
                  "radians": -0.08349884771887339,
                  "degrees": -4.784131568497007,
                  "percentage": -8.369344335413745,
                  "degMinSec": {}
                },
                "radians": -0.08349884771887339,
                "degrees": -4.784131568497007,
                "degMinSec": {},
                "vrc": "E004.47.002.874",
                "nats": "0044703W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "percentage": 143.80375687141498,
                  "degMinSec": {}
                },
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "degMinSec": {},
                "vrc": "N055.11.007.990",
                "nats": "551108N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "percentage": -8.566514145879498,
                  "degMinSec": {}
                },
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "degMinSec": {},
                "vrc": "E004.53.046.670",
                "nats": "0045347W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.605435815795432,
              "degrees": 206.57625555038516,
              "percentage": 50.024446541215454,
              "degMinSec": {}
            },
            "radians": 3.605435815795432,
            "degrees": 206.57625555038516
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 3.601680291892891,
              "degrees": 206.36107988090907,
              "percentage": 49.55579246914304,
              "degMinSec": {}
            },
            "radians": 3.601680291892891,
            "degrees": 206.36107988090907
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BLACA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "GIRVA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 37540.22363486868,
            "feet": 123163.46731022255,
            "nauticalMiles": 20.270099154896695,
            "statuteMiles": 23.326413516854494
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 4.374775767884624,
              "degrees": 250.65618781589282,
              "percentage": 284.8567102221991,
              "degMinSec": {}
            },
            "radians": 4.374775767884624,
            "degrees": 250.65618781589282
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 4.359504454298796,
              "degrees": 249.7812059998041,
              "percentage": 271.51717507965094,
              "degMinSec": {}
            },
            "radians": 4.359504454298796,
            "degrees": 249.7812059998041
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BLACA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 73010.54883674304,
            "feet": 239535.92904554002,
            "nauticalMiles": 39.422542568435766,
            "statuteMiles": 45.366651776589116
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> GIRVA; GIRVA =(TF)=> BLACA; BLACA =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9654025891289202,
                "degrees": 55.31349388808942,
                "percentage": 144.49103668203495,
                "degMinSec": {}
              },
              "radians": 0.9654025891289202,
              "degrees": 55.31349388808942,
              "degMinSec": {},
              "vrc": "N055.18.048.578",
              "nats": "551849N"
            },
            "lon": {
              "angle": {
                "radians": -0.08349884771887339,
                "degrees": -4.784131568497007,
                "percentage": -8.369344335413745,
                "degMinSec": {}
              },
              "radians": -0.08349884771887339,
              "degrees": -4.784131568497007,
              "degMinSec": {},
              "vrc": "E004.47.002.874",
              "nats": "0044703W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "percentage": 143.80375687141498,
                "degMinSec": {}
              },
              "radians": 0.96316959554411,
              "degrees": 55.18555278,
              "degMinSec": {},
              "vrc": "N055.11.007.990",
              "nats": "551108N"
            },
            "lon": {
              "angle": {
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "percentage": -8.566514145879498,
                "degMinSec": {}
              },
              "radians": -0.0854565076452456,
              "degrees": -4.896297220000025,
              "degMinSec": {},
              "vrc": "E004.53.046.670",
              "nats": "0045347W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "percentage": 143.80375687141498,
                "degMinSec": {}
              },
              "radians": 0.96316959554411,
              "degrees": 55.18555278,
              "degMinSec": {},
              "vrc": "N055.11.007.990",
              "nats": "551108N"
            },
            "lon": {
              "angle": {
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "percentage": -8.566514145879498,
                "degMinSec": {}
              },
              "radians": -0.0854565076452456,
              "degrees": -4.896297220000025,
              "degMinSec": {},
              "vrc": "E004.53.046.670",
              "nats": "0045347W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 325.5146683471649,
      "crossTrackDistance_m": 7.766587179043098,
      "requiredTrueCourse": 206.50805851862478,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 211,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 3.6826447217080354,
          "degrees": 211,
          "percentage": 60.086061902756036,
          "degMinSec": {}
        },
        "radians": 3.6826447217080354,
        "degrees": 211
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY815Z",
    "delayMs": 1860000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A320",
      "filedTas": 400,
      "origin": "EGGD",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 34000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "BCN P69 DIZIM N864 KISWO DCT AVTIC DCT MONTY UN864 BILVO UP6 TUPEM DCT REMSI DCT UVPOK M147 ROBOP M146 IPSET P6 BELZU DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9432721361223189,
          "degrees": 54.04551233209855,
          "percentage": 137.86835968394615,
          "degMinSec": {}
        },
        "radians": 0.9432721361223189,
        "degrees": 54.04551233209855,
        "degMinSec": {},
        "vrc": "N054.02.043.844",
        "nats": "540244N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07561584596378701,
          "degrees": -4.332468838036337,
          "percentage": -7.576029399660785,
          "degMinSec": {}
        },
        "radians": -0.07561584596378701,
        "degrees": -4.332468838036337,
        "degMinSec": {},
        "vrc": "E004.19.056.888",
        "nats": "0041957W"
      },
      "indicatedAltitude": {
        "meters": 6580.056172883153,
        "feet": 21588.111494241963,
        "nauticalMiles": 3.5529460976690888,
        "statuteMiles": 4.088657349133033
      },
      "trueAltitude": {
        "meters": 6719.092121633153,
        "feet": 22044.26619633891,
        "nauticalMiles": 3.6280195041215726,
        "statuteMiles": 4.17505028237167
      },
      "pressureAltitude": {
        "meters": 6580.056172883153,
        "feet": 21588.111494241963,
        "nauticalMiles": 3.5529460976690888,
        "statuteMiles": 4.088657349133033
      },
      "densityAltitude": {
        "meters": 6687.778642761971,
        "feet": 21941.531682319186,
        "nauticalMiles": 3.6111115781652114,
        "statuteMiles": 4.155592988672385
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.310934279687999,
          "degrees": 304.29411949747424,
          "percentage": -146.62684216983084,
          "degMinSec": {}
        },
        "radians": 5.310934279687999,
        "degrees": 304.29411949747424
      },
      "heading_True": {
        "angle": {
          "radians": 5.297110975275142,
          "degrees": 303.502102495693,
          "percentage": -151.07147428918447,
          "degMinSec": {}
        },
        "radians": 5.297110975275142,
        "degrees": 303.502102495693
      },
      "track_True": {
        "angle": {
          "radians": 5.321064707844121,
          "degrees": 304.87455027548054,
          "percentage": -143.48241449045608,
          "degMinSec": {}
        },
        "radians": 5.321064707844121,
        "degrees": 304.87455027548054
      },
      "track_Mag": {
        "angle": {
          "radians": 5.334888012256975,
          "degrees": 305.66656727726166,
          "percentage": -139.33622377017306,
          "degMinSec": {}
        },
        "radians": 5.334888012256975,
        "degrees": 305.66656727726166
      },
      "bank": {
        "radians": 0.0014234958551935396,
        "degrees": 0.0815603046569556,
        "percentage": 0.14234968166900627,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.049684570662783215,
        "degrees": -2.8467162058969855,
        "percentage": -4.972549412966103,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 145.66795981851467,
        "knots": 283.15578968546083,
        "feetPerMinute": 28674.796157458542
      },
      "trueAirSpeed": {
        "metersPerSecond": 200.96234007971262,
        "knots": 390.63943898990885,
        "feetPerMinute": 39559.51702962746
      },
      "groundSpeed": {
        "metersPerSecond": 202.33627026023373,
        "knots": 393.31014492773375,
        "feetPerMinute": 39829.975735235115
      },
      "machNumber": 0.6303296020997742,
      "verticalSpeed": {
        "metersPerSecond": -15.448755244643769,
        "knots": -30.02997018976932,
        "feetPerMinute": -3041.093649410224
      },
      "flightPathAngle": {
        "radians": -0.07620403262261878,
        "degrees": -4.366169451153298,
        "percentage": -7.6351883054750544,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -28.247231178662325,
        "knots": -54.90821084325569,
        "feetPerMinute": -5560.47875641215
      },
      "velocity_Y": {
        "metersPerSecond": -15.448755244643769,
        "knots": -30.02997018976932,
        "feetPerMinute": -3041.093649410224
      },
      "velocity_Z": {
        "metersPerSecond": -200.3548357129459,
        "knots": -389.4585452715956,
        "feetPerMinute": -39439.929552027694
      },
      "heading_Velocity": {
        "radiansPerSecond": 6.898952721412503E-05,
        "degreesPerSecond": 0.0039528087399723
      },
      "bank_Velocity": {
        "radiansPerSecond": -0.0019014171490068475,
        "degreesPerSecond": -0.10894317773188993
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0.004958485644455473,
        "degreesPerSecond": 0.2841003002035047
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.36789195482647874,
        "knotsPerSecond": 0.7151245690377217
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.448342192379961,
          "degrees": 197.57545394026116,
          "percentage": 31.674723020082062,
          "degMinSec": {}
        },
        "radians": 3.448342192379961,
        "degrees": 197.57545394026116
      },
      "windSpeed": {
        "metersPerSecond": 5.006915585752202,
        "knots": 9.732662819870901,
        "feetPerMinute": 985.6133358215552
      },
      "windXComp": {
        "metersPerSecond": 4.814719051190995,
        "knots": 9.359062739343308,
        "feetPerMinute": 947.7793711145679
      },
      "windHComp": {
        "metersPerSecond": -1.3739301805211206,
        "knots": -2.670705937824897,
        "feetPerMinute": -270.4587056076548
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 6611.0947265625,
          "feet": 21689.944022695312,
          "nauticalMiles": 3.5697055758976783,
          "statuteMiles": 4.107943812238092
        },
        "levelPressure": {
          "pascals": 45000,
          "hectopascals": 450,
          "inchesOfMercury": 13.290017720023627
        },
        "temp": {
          "kelvin": 253.64968872070312,
          "celsius": -19.500311279296852
        },
        "v": {
          "metersPerSecond": 4.773193359375,
          "knots": 9.278343272460937,
          "feetPerMinute": 939.6050220703125
        },
        "u": {
          "metersPerSecond": 1.511895775794983,
          "knots": 2.9388895324044224,
          "feetPerMinute": 297.6172882235527
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9432721361223189,
            "degrees": 54.04551233209855,
            "percentage": 137.86835968394615,
            "degMinSec": {}
          },
          "radians": 0.9432721361223189,
          "degrees": 54.04551233209855,
          "degMinSec": {},
          "vrc": "N054.02.043.844",
          "nats": "540244N"
        },
        "lon": {
          "angle": {
            "radians": -0.07561584596378701,
            "degrees": -4.332468838036337,
            "percentage": -7.576029399660785,
            "degMinSec": {}
          },
          "radians": -0.07561584596378701,
          "degrees": -4.332468838036337,
          "degMinSec": {},
          "vrc": "E004.19.056.888",
          "nats": "0041957W"
        },
        "alt": {
          "meters": 6719.092121633153,
          "feet": 22044.26619633891,
          "nauticalMiles": 3.6280195041215726,
          "statuteMiles": 4.17505028237167
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": -0.0019014171490068475,
        "degreesPerSecond": -0.10894317773188993
      },
      "pitchRate": {
        "radiansPerSecond": 0.004958485644455473,
        "degreesPerSecond": 0.2841003002035047
      },
      "yawRate": {
        "radiansPerSecond": 6.898952721412503E-05,
        "degreesPerSecond": 0.0039528087399723
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 34000,
      "departureAirport": {
        "identifier": "EGGD",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.8967986510815825,
              "degrees": 51.38277778,
              "percentage": 125.1906391980188,
              "degMinSec": {}
            },
            "radians": 0.8967986510815825,
            "degrees": 51.38277778,
            "degMinSec": {},
            "vrc": "N051.22.058.000",
            "nats": "512258N"
          },
          "lon": {
            "angle": {
              "radians": -0.047458411301990466,
              "degrees": -2.71916667000002,
              "percentage": -4.749407363722402,
              "degMinSec": {}
            },
            "radians": -0.047458411301990466,
            "degrees": -2.71916667000002,
            "degMinSec": {},
            "vrc": "E002.43.009.000",
            "nats": "0024309W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9423822077024802,
                  "degrees": 53.99452318957305,
                  "percentage": 137.61052832762365,
                  "degMinSec": {}
                },
                "radians": 0.9423822077024802,
                "degrees": 53.99452318957305,
                "degMinSec": {},
                "vrc": "N053.59.040.283",
                "nats": "535940N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07346130198180667,
                  "degrees": -4.209022561093552,
                  "percentage": -7.359373403900564,
                  "degMinSec": {}
                },
                "radians": -0.07346130198180667,
                "degrees": -4.209022561093552,
                "degMinSec": {},
                "vrc": "E004.12.032.481",
                "nats": "0041232W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "percentage": 138.0824602372339,
                  "degMinSec": {}
                },
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "degMinSec": {},
                "vrc": "N054.05.015.930",
                "nats": "540516N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "percentage": -7.75714864160994,
                  "degMinSec": {}
                },
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "degMinSec": {},
                "vrc": "E004.26.008.290",
                "nats": "0042608W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "NOPKI"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.3251548857506315,
            "degrees": 305.1089002069813,
            "percentage": -142.2386358704023,
            "degMinSec": {}
          },
          "radians": 5.3251548857506315,
          "degrees": 305.1089002069813
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.321953429152938,
            "degrees": 304.9254702556391,
            "percentage": -143.21092561718953,
            "degMinSec": {}
          },
          "radians": 5.321953429152938,
          "degrees": 304.9254702556391
        },
        "legLength": {
          "meters": 18067.02207589468,
          "feet": 59275.00870747831,
          "nauticalMiles": 9.755411488064082,
          "statuteMiles": 11.22632704747691
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9423822077024802,
                  "degrees": 53.99452318957305,
                  "percentage": 137.61052832762365,
                  "degMinSec": {}
                },
                "radians": 0.9423822077024802,
                "degrees": 53.99452318957305,
                "degMinSec": {},
                "vrc": "N053.59.040.283",
                "nats": "535940N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07346130198180667,
                  "degrees": -4.209022561093552,
                  "percentage": -7.359373403900564,
                  "degMinSec": {}
                },
                "radians": -0.07346130198180667,
                "degrees": -4.209022561093552,
                "degMinSec": {},
                "vrc": "E004.12.032.481",
                "nats": "0041232W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "percentage": 138.0824602372339,
                  "degMinSec": {}
                },
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "degMinSec": {},
                "vrc": "N054.05.015.930",
                "nats": "540516N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "percentage": -7.75714864160994,
                  "degMinSec": {}
                },
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "degMinSec": {},
                "vrc": "E004.26.008.290",
                "nats": "0042608W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.3257006040392305,
              "degrees": 305.1401675617211,
              "percentage": -142.07378319360646,
              "degMinSec": {}
            },
            "radians": 5.3257006040392305,
            "degrees": 305.1401675617211
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.312960633207489,
              "degrees": 304.4102210019425,
              "percentage": -145.99044233056168,
              "degMinSec": {}
            },
            "radians": 5.312960633207489,
            "degrees": 304.4102210019425
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9440094678814897,
                    "degrees": 54.08775833,
                    "percentage": 138.0824602372339,
                    "degMinSec": {}
                  },
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "degMinSec": {},
                  "vrc": "N054.05.015.930",
                  "nats": "540516N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07741645453985413,
                    "degrees": -4.435636110000043,
                    "percentage": -7.75714864160994,
                    "degMinSec": {}
                  },
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "degMinSec": {},
                  "vrc": "E004.26.008.290",
                  "nats": "0042608W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "NOPKI"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 71082.94499483609,
            "feet": 233211.769256858,
            "nauticalMiles": 38.38171975963071,
            "statuteMiles": 44.168894279182126
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9440094678814897,
                    "degrees": 54.08775833,
                    "percentage": 138.0824602372339,
                    "degMinSec": {}
                  },
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "degMinSec": {},
                  "vrc": "N054.05.015.930",
                  "nats": "540516N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07741645453985413,
                    "degrees": -4.435636110000043,
                    "percentage": -7.75714864160994,
                    "degMinSec": {}
                  },
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "degMinSec": {},
                  "vrc": "E004.26.008.290",
                  "nats": "0042608W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.480023517854051,
              "degrees": 313.9822192054715,
              "percentage": -103.617363161653,
              "degMinSec": {}
            },
            "radians": 5.480023517854051,
            "degrees": 313.9822192054715
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.478352016380683,
              "degrees": 313.8864492255976,
              "percentage": -103.96457671864067,
              "degMinSec": {}
            },
            "radians": 5.478352016380683,
            "degrees": 313.8864492255976
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 10553.672770561494,
            "feet": 34624.91177256897,
            "nauticalMiles": 5.698527413910094,
            "statuteMiles": 6.557748231926483
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.1474641895474593,
              "degrees": 180.3364142296336,
              "percentage": 0.5871603432202414,
              "degMinSec": {}
            },
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 6.165737882555982,
              "degrees": 353.27075825438664,
              "percentage": -11.799044018857247,
              "degMinSec": {}
            },
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16253794.83622146,
            "feet": 53326100.25046881,
            "nauticalMiles": 8776.347103791286,
            "statuteMiles": 10099.63987576395
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> NOPKI; NOPKI =(TF)=> ROBOP; ROBOP =(TF)=> IPSET; IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9423822077024802,
                "degrees": 53.99452318957305,
                "percentage": 137.61052832762365,
                "degMinSec": {}
              },
              "radians": 0.9423822077024802,
              "degrees": 53.99452318957305,
              "degMinSec": {},
              "vrc": "N053.59.040.283",
              "nats": "535940N"
            },
            "lon": {
              "angle": {
                "radians": -0.07346130198180667,
                "degrees": -4.209022561093552,
                "percentage": -7.359373403900564,
                "degMinSec": {}
              },
              "radians": -0.07346130198180667,
              "degrees": -4.209022561093552,
              "degMinSec": {},
              "vrc": "E004.12.032.481",
              "nats": "0041232W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "percentage": 138.0824602372339,
                "degMinSec": {}
              },
              "radians": 0.9440094678814897,
              "degrees": 54.08775833,
              "degMinSec": {},
              "vrc": "N054.05.015.930",
              "nats": "540516N"
            },
            "lon": {
              "angle": {
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "percentage": -7.75714864160994,
                "degMinSec": {}
              },
              "radians": -0.07741645453985413,
              "degrees": -4.435636110000043,
              "degMinSec": {},
              "vrc": "E004.26.008.290",
              "nats": "0042608W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "percentage": 138.0824602372339,
                "degMinSec": {}
              },
              "radians": 0.9440094678814897,
              "degrees": 54.08775833,
              "degMinSec": {},
              "vrc": "N054.05.015.930",
              "nats": "540516N"
            },
            "lon": {
              "angle": {
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "percentage": -7.75714864160994,
                "degMinSec": {}
              },
              "radians": -0.07741645453985413,
              "degrees": -4.435636110000043,
              "degMinSec": {},
              "vrc": "E004.26.008.290",
              "nats": "0042608W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 9016.520037220564,
      "crossTrackDistance_m": 10.807875989599648,
      "requiredTrueCourse": 305.01704529393083,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "CFE95C",
    "delayMs": 1920000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "E190",
      "filedTas": 440,
      "origin": "EGLC",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 40000,
      "destination": "EGAC",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "BPK N601 LESTA P6 RODOL L28 AGLIL M146 ERDUV Z198 MASOP M148 MAGEE DCT"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9441195142074068,
          "degrees": 54.094063520025976,
          "percentage": 138.11445200651545,
          "degMinSec": {}
        },
        "radians": 0.9441195142074068,
        "degrees": 54.094063520025976,
        "degMinSec": {},
        "vrc": "N054.05.038.629",
        "nats": "540539N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07336529568150141,
          "degrees": -4.2035218052793955,
          "percentage": -7.3497208446660816,
          "degMinSec": {}
        },
        "radians": -0.07336529568150141,
        "degrees": -4.2035218052793955,
        "degMinSec": {},
        "vrc": "E004.12.012.678",
        "nats": "0041213W"
      },
      "indicatedAltitude": {
        "meters": 6633.660718188276,
        "feet": 21763.979430660824,
        "nauticalMiles": 3.581890236602741,
        "statuteMiles": 4.121965669358619
      },
      "trueAltitude": {
        "meters": 6772.696666938276,
        "feet": 22220.134132757776,
        "nauticalMiles": 3.656963643055225,
        "statuteMiles": 4.208358602597254
      },
      "pressureAltitude": {
        "meters": 6633.660718188276,
        "feet": 21763.979430660824,
        "nauticalMiles": 3.581890236602741,
        "statuteMiles": 4.121965669358619
      },
      "densityAltitude": {
        "meters": 6739.852819355885,
        "feet": 22112.378723855563,
        "nauticalMiles": 3.6392293841014496,
        "statuteMiles": 4.1879503818673225
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.305781695338492,
          "degrees": 303.99889816066235,
          "percentage": -148.26224698728987,
          "degMinSec": {}
        },
        "radians": 5.305781695338492,
        "degrees": 303.99889816066235
      },
      "heading_True": {
        "angle": {
          "radians": 5.292630147178549,
          "degrees": 303.2453699570346,
          "percentage": -152.55223167093035,
          "degMinSec": {}
        },
        "radians": 5.292630147178549,
        "degrees": 303.2453699570346
      },
      "track_True": {
        "angle": {
          "radians": 5.317566127786748,
          "degrees": 304.67409640390446,
          "percentage": -144.55793562393617,
          "degMinSec": {}
        },
        "radians": 5.317566127786748,
        "degrees": 304.67409640390446
      },
      "track_Mag": {
        "angle": {
          "radians": 5.3307176759466905,
          "degrees": 305.4276246075322,
          "percentage": -140.57008769223336,
          "degMinSec": {}
        },
        "radians": 5.3307176759466905,
        "degrees": 305.4276246075322
      },
      "bank": {
        "radians": 0.01169418712968033,
        "degrees": 0.6700275673668893,
        "percentage": 1.1694720234513492,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.07756493117664412,
        "degrees": -4.444143194644407,
        "percentage": -7.772085820812283,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 139.4476339653311,
        "knots": 271.06444659770506,
        "feetPerMinute": 27450.322525129013
      },
      "trueAirSpeed": {
        "metersPerSecond": 193.2880793269199,
        "knots": 375.7218732711573,
        "feetPerMinute": 38048.83573073591
      },
      "groundSpeed": {
        "metersPerSecond": 194.64042185849343,
        "knots": 378.35061618710125,
        "feetPerMinute": 38315.04489901317
      },
      "machNumber": 0.6066871453579175,
      "verticalSpeed": {
        "metersPerSecond": -22.02784153588937,
        "knots": -42.818687602489334,
        "feetPerMinute": -4336.189417476437
      },
      "flightPathAngle": {
        "radians": -0.11269249234206627,
        "degrees": -6.456804194010747,
        "percentage": -11.31719779764141,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 11.747384975127119,
        "knots": 22.835083799590997,
        "feetPerMinute": 2312.4774313077633
      },
      "velocity_Y": {
        "metersPerSecond": -22.02784153588937,
        "knots": -42.818687602489334,
        "feetPerMinute": -4336.189417476437
      },
      "velocity_Z": {
        "metersPerSecond": -194.28559588270676,
        "knots": -377.6608898430242,
        "feetPerMinute": -38245.19726374918
      },
      "heading_Velocity": {
        "radiansPerSecond": 0.0005890706861701059,
        "degreesPerSecond": 0.0337512641524225
      },
      "bank_Velocity": {
        "radiansPerSecond": -0.016421647648009966,
        "degreesPerSecond": -0.9408911028819059
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0.0051342000854923955,
        "degreesPerSecond": 0.2941679960744207
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 1.0438936657885596,
        "knotsPerSecond": 2.029166438881097
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.448342192379961,
          "degrees": 197.57545394026116,
          "percentage": 31.674723020082062,
          "degMinSec": {}
        },
        "radians": 3.448342192379961,
        "degrees": 197.57545394026116
      },
      "windSpeed": {
        "metersPerSecond": 5.006915585752202,
        "knots": 9.732662819870901,
        "feetPerMinute": 985.6133358215552
      },
      "windXComp": {
        "metersPerSecond": 4.820827041094256,
        "knots": 9.370935718868823,
        "feetPerMinute": 948.9817313702207
      },
      "windHComp": {
        "metersPerSecond": -1.3523425315735322,
        "knots": -2.628742915944021,
        "feetPerMinute": -266.20916827726245
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 6611.0947265625,
          "feet": 21689.944022695312,
          "nauticalMiles": 3.5697055758976783,
          "statuteMiles": 4.107943812238092
        },
        "levelPressure": {
          "pascals": 45000,
          "hectopascals": 450,
          "inchesOfMercury": 13.290017720023627
        },
        "temp": {
          "kelvin": 253.64968872070312,
          "celsius": -19.500311279296852
        },
        "v": {
          "metersPerSecond": 4.773193359375,
          "knots": 9.278343272460937,
          "feetPerMinute": 939.6050220703125
        },
        "u": {
          "metersPerSecond": 1.511895775794983,
          "knots": 2.9388895324044224,
          "feetPerMinute": 297.6172882235527
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9441195142074068,
            "degrees": 54.094063520025976,
            "percentage": 138.11445200651545,
            "degMinSec": {}
          },
          "radians": 0.9441195142074068,
          "degrees": 54.094063520025976,
          "degMinSec": {},
          "vrc": "N054.05.038.629",
          "nats": "540539N"
        },
        "lon": {
          "angle": {
            "radians": -0.07336529568150141,
            "degrees": -4.2035218052793955,
            "percentage": -7.3497208446660816,
            "degMinSec": {}
          },
          "radians": -0.07336529568150141,
          "degrees": -4.2035218052793955,
          "degMinSec": {},
          "vrc": "E004.12.012.678",
          "nats": "0041213W"
        },
        "alt": {
          "meters": 6772.696666938276,
          "feet": 22220.134132757776,
          "nauticalMiles": 3.656963643055225,
          "statuteMiles": 4.208358602597254
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": -0.016421647648009966,
        "degreesPerSecond": -0.9408911028819059
      },
      "pitchRate": {
        "radiansPerSecond": 0.0051342000854923955,
        "degreesPerSecond": 0.2941679960744207
      },
      "yawRate": {
        "radiansPerSecond": 0.0005890706861701059,
        "degreesPerSecond": 0.0337512641524225
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 40000,
      "departureAirport": {
        "identifier": "EGLC",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.8989366794152757,
              "degrees": 51.50527778,
              "percentage": 125.74100266326668,
              "degMinSec": {}
            },
            "radians": 0.8989366794152757,
            "degrees": 51.50527778,
            "degMinSec": {},
            "vrc": "N051.30.019.000",
            "nats": "513019N"
          },
          "lon": {
            "angle": {
              "radians": 0.0009647792641924724,
              "degrees": 0.0552777799999657,
              "percentage": 0.09647795635311178,
              "degMinSec": {}
            },
            "radians": 0.0009647792641924724,
            "degrees": 0.0552777799999657,
            "degMinSec": {},
            "vrc": "E000.03.019.000",
            "nats": "0000319E"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAC",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9532649005591953,
              "degrees": 54.61805556,
              "percentage": 140.80762090061415,
              "degMinSec": {}
            },
            "radians": 0.9532649005591953,
            "degrees": 54.61805556,
            "degMinSec": {},
            "vrc": "N054.37.005.000",
            "nats": "543705N"
          },
          "lon": {
            "angle": {
              "radians": -0.10249446032336706,
              "degrees": -5.872500000000004,
              "percentage": -10.285488024374672,
              "degMinSec": {}
            },
            "radians": -0.10249446032336706,
            "degrees": -5.872500000000004,
            "degMinSec": {},
            "vrc": "E005.52.021.000",
            "nats": "0055221W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9435682474648426,
                  "degrees": 54.062478262291116,
                  "percentage": 137.95428981121358,
                  "degMinSec": {}
                },
                "radians": 0.9435682474648426,
                "degrees": 54.062478262291116,
                "degMinSec": {},
                "vrc": "N054.03.044.922",
                "nats": "540345N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07204840731193585,
                  "degrees": -4.128069659613424,
                  "percentage": -7.217333382667329,
                  "degMinSec": {}
                },
                "radians": -0.07204840731193585,
                "degrees": -4.128069659613424,
                "degMinSec": {},
                "vrc": "E004.07.041.051",
                "nats": "0040741W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "percentage": 138.4284080202272,
                  "degMinSec": {}
                },
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "degMinSec": {},
                "vrc": "N054.09.021.020",
                "nats": "540921N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "percentage": -7.6142444922198695,
                  "degMinSec": {}
                },
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "degMinSec": {},
                "vrc": "E004.21.015.260",
                "nats": "0042115W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MASOP"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.3274794921425315,
            "degrees": 305.2420903422663,
            "percentage": -141.5381796174425,
            "degMinSec": {}
          },
          "radians": 5.3274794921425315,
          "degrees": 305.2420903422663
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.324281563166741,
            "degrees": 305.05886250877086,
            "percentage": -142.50298570838612,
            "degMinSec": {}
          },
          "radians": 5.324281563166741,
          "degrees": 305.05886250877086
        },
        "legLength": {
          "meters": 18031.518139870503,
          "feet": 59158.52597401274,
          "nauticalMiles": 9.736240896258371,
          "statuteMiles": 11.204265924420449
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9435682474648426,
                  "degrees": 54.062478262291116,
                  "percentage": 137.95428981121358,
                  "degMinSec": {}
                },
                "radians": 0.9435682474648426,
                "degrees": 54.062478262291116,
                "degMinSec": {},
                "vrc": "N054.03.044.922",
                "nats": "540345N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07204840731193585,
                  "degrees": -4.128069659613424,
                  "percentage": -7.217333382667329,
                  "degMinSec": {}
                },
                "radians": -0.07204840731193585,
                "degrees": -4.128069659613424,
                "degMinSec": {},
                "vrc": "E004.07.041.051",
                "nats": "0040741W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "percentage": 138.4284080202272,
                  "degMinSec": {}
                },
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "degMinSec": {},
                "vrc": "N054.09.021.020",
                "nats": "540921N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "percentage": -7.6142444922198695,
                  "degMinSec": {}
                },
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "degMinSec": {},
                "vrc": "E004.21.015.260",
                "nats": "0042115W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.438593895987405,
              "degrees": 311.60847672568974,
              "percentage": -112.5991529192575,
              "degMinSec": {}
            },
            "radians": 5.438593895987405,
            "degrees": 311.60847672568974
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.420779718567154,
              "degrees": 310.5877995440121,
              "percentage": -116.72229413505232,
              "degMinSec": {}
            },
            "radians": 5.420779718567154,
            "degrees": 310.5877995440121
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAGEE"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9451976978100912,
                    "degrees": 54.15583889,
                    "percentage": 138.4284080202272,
                    "degMinSec": {}
                  },
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "degMinSec": {},
                  "vrc": "N054.09.021.020",
                  "nats": "540921N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07599580504888337,
                    "degrees": -4.35423889000001,
                    "percentage": -7.6142444922198695,
                    "degMinSec": {}
                  },
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "degMinSec": {},
                  "vrc": "E004.21.015.260",
                  "nats": "0042115W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MASOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 107526.38245727522,
            "feet": 352776.8566211268,
            "nauticalMiles": 58.05960175878791,
            "statuteMiles": 66.81379646444465
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9451976978100912,
                    "degrees": 54.15583889,
                    "percentage": 138.4284080202272,
                    "degMinSec": {}
                  },
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "degMinSec": {},
                  "vrc": "N054.09.021.020",
                  "nats": "540921N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07599580504888337,
                    "degrees": -4.35423889000001,
                    "percentage": -7.6142444922198695,
                    "degMinSec": {}
                  },
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "degMinSec": {},
                  "vrc": "E004.21.015.260",
                  "nats": "0042115W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> MASOP; MASOP =(TF)=> MAGEE; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9435682474648426,
                "degrees": 54.062478262291116,
                "percentage": 137.95428981121358,
                "degMinSec": {}
              },
              "radians": 0.9435682474648426,
              "degrees": 54.062478262291116,
              "degMinSec": {},
              "vrc": "N054.03.044.922",
              "nats": "540345N"
            },
            "lon": {
              "angle": {
                "radians": -0.07204840731193585,
                "degrees": -4.128069659613424,
                "percentage": -7.217333382667329,
                "degMinSec": {}
              },
              "radians": -0.07204840731193585,
              "degrees": -4.128069659613424,
              "degMinSec": {},
              "vrc": "E004.07.041.051",
              "nats": "0040741W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "percentage": 138.4284080202272,
                "degMinSec": {}
              },
              "radians": 0.9451976978100912,
              "degrees": 54.15583889,
              "degMinSec": {},
              "vrc": "N054.09.021.020",
              "nats": "540921N"
            },
            "lon": {
              "angle": {
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "percentage": -7.6142444922198695,
                "degMinSec": {}
              },
              "radians": -0.07599580504888337,
              "degrees": -4.35423889000001,
              "degMinSec": {},
              "vrc": "E004.21.015.260",
              "nats": "0042115W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "percentage": 138.4284080202272,
                "degMinSec": {}
              },
              "radians": 0.9451976978100912,
              "degrees": 54.15583889,
              "degMinSec": {},
              "vrc": "N054.09.021.020",
              "nats": "540921N"
            },
            "lon": {
              "angle": {
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "percentage": -7.6142444922198695,
                "degMinSec": {}
              },
              "radians": -0.07599580504888337,
              "degrees": -4.35423889000001,
              "degMinSec": {},
              "vrc": "E004.21.015.260",
              "nats": "0042115W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 12758.188820428704,
      "crossTrackDistance_m": 39.42881737376763,
      "requiredTrueCourse": 305.1884735538619,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY421",
    "delayMs": 1920000,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "IFR",
      "aircraftType": "A320",
      "filedTas": 340,
      "origin": "EGPF",
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 20000,
      "destination": "EGAA",
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": "",
      "remarks": "",
      "route": "NORBO L186 TRN P600 BLACA DCT BELZU"
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9646185070775489,
          "degrees": 55.268569295753885,
          "percentage": 144.24920442363657,
          "degMinSec": {}
        },
        "radians": 0.9646185070775489,
        "degrees": 55.268569295753885,
        "degMinSec": {},
        "vrc": "N055.16.006.849",
        "nats": "551607N"
      },
      "longitude": {
        "angle": {
          "radians": -0.08419122994524564,
          "degrees": -4.823802147878008,
          "percentage": -8.439071595270468,
          "degMinSec": {}
        },
        "radians": -0.08419122994524564,
        "degrees": -4.823802147878008,
        "degMinSec": {},
        "vrc": "E004.49.025.688",
        "nats": "0044926W"
      },
      "indicatedAltitude": {
        "meters": 5612.465032889038,
        "feet": 18413.599778503674,
        "nauticalMiles": 3.0304886786657876,
        "statuteMiles": 3.4874240888766095
      },
      "trueAltitude": {
        "meters": 5557.794628514039,
        "feet": 18234.234929013997,
        "nauticalMiles": 3.000969021875831,
        "statuteMiles": 3.453453474530019
      },
      "pressureAltitude": {
        "meters": 5612.465032889038,
        "feet": 18413.599778503674,
        "nauticalMiles": 3.0304886786657876,
        "statuteMiles": 3.4874240888766095
      },
      "densityAltitude": {
        "meters": 5573.155569214234,
        "feet": 18284.631717700828,
        "nauticalMiles": 3.009263266314381,
        "statuteMiles": 3.462998320566786
      },
      "heading_Mag": {
        "angle": {
          "radians": 3.613810131451938,
          "degrees": 207.05606849381329,
          "percentage": 51.075869978357744,
          "degMinSec": {}
        },
        "radians": 3.613810131451938,
        "degrees": 207.05606849381329
      },
      "heading_True": {
        "angle": {
          "radians": 3.593482576745167,
          "degrees": 205.89138540129403,
          "percentage": 48.53881203016683,
          "degMinSec": {}
        },
        "radians": 3.593482576745167,
        "degrees": 205.89138540129403
      },
      "track_True": {
        "angle": {
          "radians": 3.600647463547647,
          "degrees": 206.3019031757651,
          "percentage": 49.42721144356252,
          "degMinSec": {}
        },
        "radians": 3.600647463547647,
        "degrees": 206.3019031757651
      },
      "track_Mag": {
        "angle": {
          "radians": 3.620975018254418,
          "degrees": 207.46658626828437,
          "percentage": 51.98260599063773,
          "degMinSec": {}
        },
        "radians": 3.620975018254418,
        "degrees": 207.46658626828437
      },
      "bank": {
        "radians": 0.005934185776952803,
        "degrees": 0.3400037998659569,
        "percentage": 0.5934255434516004,
        "degMinSec": {}
      },
      "pitch": {
        "radians": -0.07860799448231554,
        "degrees": -4.503906319824343,
        "percentage": -7.877030762798566,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 140.37568496471005,
        "knots": 272.8684329645418,
        "feetPerMinute": 27633.009735577158
      },
      "trueAirSpeed": {
        "metersPerSecond": 183.41371779965147,
        "knots": 356.52765486254566,
        "feetPerMinute": 36105.063714348515
      },
      "groundSpeed": {
        "metersPerSecond": 173.92746304911992,
        "knots": 338.08785548325346,
        "feetPerMinute": 34237.69067220447
      },
      "machNumber": 0.5670165679164425,
      "verticalSpeed": {
        "metersPerSecond": -20.479991411002068,
        "knots": -39.8099084243279,
        "feetPerMinute": -4031.4945012523217
      },
      "flightPathAngle": {
        "radians": -0.11721046427066915,
        "degrees": -6.715664917478273,
        "percentage": -11.775018764700885,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -150.27780421531284,
        "knots": -292.11660805711057,
        "feetPerMinute": -29582.24587090602
      },
      "velocity_Y": {
        "metersPerSecond": -20.479991411002068,
        "knots": -39.8099084243279,
        "feetPerMinute": -4031.4945012523217
      },
      "velocity_Z": {
        "metersPerSecond": 87.56337112587136,
        "knots": 170.20953358279826,
        "feetPerMinute": 17236.884631476227
      },
      "heading_Velocity": {
        "radiansPerSecond": 0.0003345107223572321,
        "degreesPerSecond": 0.019166052592941865
      },
      "bank_Velocity": {
        "radiansPerSecond": -0.0076544424400298945,
        "degreesPerSecond": -0.4385672463395327
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0.005520419242422925,
        "degreesPerSecond": 0.31629672373364087
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0.9517920388645221,
        "knotsPerSecond": 1.850135243994568
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 100727.1171875,
        "hectopascals": 1007.271171875,
        "inchesOfMercury": 29.74811494019492
      },
      "windDirection": {
        "angle": {
          "radians": 3.455825549445601,
          "degrees": 198.00421871671173,
          "percentage": 32.50011020558129,
          "degMinSec": {}
        },
        "radians": 3.455825549445601,
        "degrees": 198.00421871671173
      },
      "windSpeed": {
        "metersPerSecond": 9.576849605292526,
        "knots": 18.615901644150245,
        "feetPerMinute": 1885.206675541676
      },
      "windXComp": {
        "metersPerSecond": 1.3141610138827728,
        "knots": 2.5545240018699444,
        "feetPerMinute": 258.6931212472294
      },
      "windHComp": {
        "metersPerSecond": 9.486254750531542,
        "knots": 18.439799379292232,
        "feetPerMinute": 1867.3730421440341
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.964294411726867,
            "degrees": 55.25,
            "percentage": 144.1494043228611,
            "degMinSec": {}
          },
          "radians": 0.964294411726867,
          "degrees": 55.25,
          "degMinSec": {},
          "vrc": "N055.15.000.000",
          "nats": "551500N"
        },
        "lon": {
          "angle": {
            "radians": -0.08290367210194916,
            "degrees": -4.7500305175781525,
            "percentage": -8.309412855705487,
            "degMinSec": {}
          },
          "radians": -0.08290367210194916,
          "degrees": -4.7500305175781525,
          "degMinSec": {},
          "vrc": "E004.45.000.110",
          "nats": "0044500W"
        },
        "geoPotentialHeight": {
          "meters": 5811.03125,
          "feet": 19065.06376625,
          "nauticalMiles": 3.1377058585313176,
          "statuteMiles": 3.610807415940905
        },
        "levelPressure": {
          "pascals": 50000,
          "hectopascals": 500,
          "inchesOfMercury": 14.766686355581808
        },
        "temp": {
          "kelvin": 258.7406921386719,
          "celsius": -14.409307861328102
        },
        "v": {
          "metersPerSecond": 9.10790729522705,
          "knots": 17.70435094838333,
          "feetPerMinute": 1792.895194228363
        },
        "u": {
          "metersPerSecond": 2.9600799083709717,
          "knots": 5.7539335694074625,
          "feetPerMinute": 582.6929139947891
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 100727.1171875,
          "hectopascals": 1007.271171875,
          "inchesOfMercury": 29.74811494019492
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9646185070775489,
            "degrees": 55.268569295753885,
            "percentage": 144.24920442363657,
            "degMinSec": {}
          },
          "radians": 0.9646185070775489,
          "degrees": 55.268569295753885,
          "degMinSec": {},
          "vrc": "N055.16.006.849",
          "nats": "551607N"
        },
        "lon": {
          "angle": {
            "radians": -0.08419122994524564,
            "degrees": -4.823802147878008,
            "percentage": -8.439071595270468,
            "degMinSec": {}
          },
          "radians": -0.08419122994524564,
          "degrees": -4.823802147878008,
          "degMinSec": {},
          "vrc": "E004.49.025.688",
          "nats": "0044926W"
        },
        "alt": {
          "meters": 5557.794628514039,
          "feet": 18234.234929013997,
          "nauticalMiles": 3.000969021875831,
          "statuteMiles": 3.453453474530019
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": -0.0076544424400298945,
        "degreesPerSecond": -0.4385672463395327
      },
      "pitchRate": {
        "radiansPerSecond": 0.005520419242422925,
        "degreesPerSecond": 0.31629672373364087
      },
      "yawRate": {
        "radiansPerSecond": 0.0003345107223572321,
        "degreesPerSecond": 0.019166052592941865
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 20000,
      "departureAirport": {
        "identifier": "EGPF",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9751493899693394,
              "degrees": 55.87194444,
              "percentage": 147.5437089010122,
              "degMinSec": {}
            },
            "radians": 0.9751493899693394,
            "degrees": 55.87194444,
            "degMinSec": {},
            "vrc": "N055.52.019.000",
            "nats": "555219N"
          },
          "lon": {
            "angle": {
              "radians": -0.0773714154458407,
              "degrees": -4.433055559999981,
              "percentage": -7.7526176465017445,
              "degMinSec": {}
            },
            "radians": -0.0773714154458407,
            "degrees": -4.433055559999981,
            "degMinSec": {},
            "vrc": "E004.25.059.000",
            "nats": "0042559W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "arrivalAirport": {
        "identifier": "EGAA",
        "location": {
          "lat": {
            "angle": {
              "radians": 0.9539533359088006,
              "degrees": 54.6575,
              "percentage": 141.01315831396087,
              "degMinSec": {}
            },
            "radians": 0.9539533359088006,
            "degrees": 54.6575,
            "degMinSec": {},
            "vrc": "N054.39.027.000",
            "nats": "543927N"
          },
          "lon": {
            "angle": {
              "radians": -0.10848675736370339,
              "degrees": -6.2158333300000095,
              "percentage": -10.891437777189294,
              "degMinSec": {}
            },
            "radians": -0.10848675736370339,
            "degrees": -6.2158333300000095,
            "degMinSec": {},
            "vrc": "E006.12.057.000",
            "nats": "0061257W"
          },
          "alt": {
            "meters": 0,
            "feet": 0,
            "nauticalMiles": 0,
            "statuteMiles": 0
          }
        },
        "name": null
      },
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9654022998705235,
                  "degrees": 55.313477314804096,
                  "percentage": 144.49094736584894,
                  "degMinSec": {}
                },
                "radians": 0.9654022998705235,
                "degrees": 55.313477314804096,
                "degMinSec": {},
                "vrc": "N055.18.048.518",
                "nats": "551849N"
              },
              "lon": {
                "angle": {
                  "radians": -0.08349783295096103,
                  "degrees": -4.784073426578443,
                  "percentage": -8.36924214782762,
                  "degMinSec": {}
                },
                "radians": -0.08349783295096103,
                "degrees": -4.784073426578443,
                "degMinSec": {},
                "vrc": "E004.47.002.664",
                "nats": "0044703W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "percentage": 143.80375687141498,
                  "degMinSec": {}
                },
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "degMinSec": {},
                "vrc": "N055.11.007.990",
                "nats": "551108N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "percentage": -8.566514145879498,
                  "degMinSec": {}
                },
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "degMinSec": {},
                "vrc": "E004.53.046.670",
                "nats": "0045347W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "GIRVA"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 3.6060808592885034,
            "degrees": 206.61321382014054,
            "percentage": 50.10511879325234,
            "degMinSec": {}
          },
          "radians": 3.6060808592885034,
          "degrees": 206.61321382014054
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 3.6044715285030655,
            "degrees": 206.52100595829447,
            "percentage": 49.9039451404594,
            "degMinSec": {}
          },
          "radians": 3.6044715285030655,
          "degrees": 206.52100595829447
        },
        "legLength": {
          "meters": 15903.81966384441,
          "feet": 52177.88770592729,
          "nauticalMiles": 8.587375628425708,
          "statuteMiles": 9.882175385650557
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9654022998705235,
                  "degrees": 55.313477314804096,
                  "percentage": 144.49094736584894,
                  "degMinSec": {}
                },
                "radians": 0.9654022998705235,
                "degrees": 55.313477314804096,
                "degMinSec": {},
                "vrc": "N055.18.048.518",
                "nats": "551849N"
              },
              "lon": {
                "angle": {
                  "radians": -0.08349783295096103,
                  "degrees": -4.784073426578443,
                  "percentage": -8.36924214782762,
                  "degMinSec": {}
                },
                "radians": -0.08349783295096103,
                "degrees": -4.784073426578443,
                "degMinSec": {},
                "vrc": "E004.47.002.664",
                "nats": "0044703W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "percentage": 143.80375687141498,
                  "degMinSec": {}
                },
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "degMinSec": {},
                "vrc": "N055.11.007.990",
                "nats": "551108N"
              },
              "lon": {
                "angle": {
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "percentage": -8.566514145879498,
                  "degMinSec": {}
                },
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "degMinSec": {},
                "vrc": "E004.53.046.670",
                "nats": "0045347W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.605435815795432,
              "degrees": 206.57625555038516,
              "percentage": 50.024446541215454,
              "degMinSec": {}
            },
            "radians": 3.605435815795432,
            "degrees": 206.57625555038516
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 3.601680291892891,
              "degrees": 206.36107988090907,
              "percentage": 49.55579246914304,
              "degMinSec": {}
            },
            "radians": 3.601680291892891,
            "degrees": 206.36107988090907
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BLACA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "GIRVA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 37540.22363486868,
            "feet": 123163.46731022255,
            "nauticalMiles": 20.270099154896695,
            "statuteMiles": 23.326413516854494
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.96316959554411,
                    "degrees": 55.18555278,
                    "percentage": 143.80375687141498,
                    "degMinSec": {}
                  },
                  "radians": 0.96316959554411,
                  "degrees": 55.18555278,
                  "degMinSec": {},
                  "vrc": "N055.11.007.990",
                  "nats": "551108N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0854565076452456,
                    "degrees": -4.896297220000025,
                    "percentage": -8.566514145879498,
                    "degMinSec": {}
                  },
                  "radians": -0.0854565076452456,
                  "degrees": -4.896297220000025,
                  "degMinSec": {},
                  "vrc": "E004.53.046.670",
                  "nats": "0045347W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 4.374775767884624,
              "degrees": 250.65618781589282,
              "percentage": 284.8567102221991,
              "degMinSec": {}
            },
            "radians": 4.374775767884624,
            "degrees": 250.65618781589282
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 4.359504454298796,
              "degrees": 249.7812059998041,
              "percentage": 271.51717507965094,
              "degMinSec": {}
            },
            "radians": 4.359504454298796,
            "degrees": 249.7812059998041
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BLACA"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 73010.54883674304,
            "feet": 239535.92904554002,
            "nauticalMiles": 39.422542568435766,
            "statuteMiles": 45.366651776589116
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9578948710780435,
                    "degrees": 54.88333333,
                    "percentage": 142.1976644186302,
                    "degMinSec": {}
                  },
                  "radians": 0.9578948710780435,
                  "degrees": 54.88333333,
                  "degMinSec": {},
                  "vrc": "N054.52.060.000",
                  "nats": "545300N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09003920906289586,
                    "degrees": -5.158866670000005,
                    "percentage": -9.028331842888736,
                    "degMinSec": {}
                  },
                  "radians": -0.09003920906289586,
                  "degrees": -5.158866670000005,
                  "degMinSec": {},
                  "vrc": "E005.09.031.920",
                  "nats": "0050932W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> GIRVA; GIRVA =(TF)=> BLACA; BLACA =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9654022998705235,
                "degrees": 55.313477314804096,
                "percentage": 144.49094736584894,
                "degMinSec": {}
              },
              "radians": 0.9654022998705235,
              "degrees": 55.313477314804096,
              "degMinSec": {},
              "vrc": "N055.18.048.518",
              "nats": "551849N"
            },
            "lon": {
              "angle": {
                "radians": -0.08349783295096103,
                "degrees": -4.784073426578443,
                "percentage": -8.36924214782762,
                "degMinSec": {}
              },
              "radians": -0.08349783295096103,
              "degrees": -4.784073426578443,
              "degMinSec": {},
              "vrc": "E004.47.002.664",
              "nats": "0044703W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "percentage": 143.80375687141498,
                "degMinSec": {}
              },
              "radians": 0.96316959554411,
              "degrees": 55.18555278,
              "degMinSec": {},
              "vrc": "N055.11.007.990",
              "nats": "551108N"
            },
            "lon": {
              "angle": {
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "percentage": -8.566514145879498,
                "degMinSec": {}
              },
              "radians": -0.0854565076452456,
              "degrees": -4.896297220000025,
              "degMinSec": {},
              "vrc": "E004.53.046.670",
              "nats": "0045347W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.96316959554411,
                "degrees": 55.18555278,
                "percentage": 143.80375687141498,
                "degMinSec": {}
              },
              "radians": 0.96316959554411,
              "degrees": 55.18555278,
              "degMinSec": {},
              "vrc": "N055.11.007.990",
              "nats": "551108N"
            },
            "lon": {
              "angle": {
                "radians": -0.0854565076452456,
                "degrees": -4.896297220000025,
                "percentage": -8.566514145879498,
                "degMinSec": {}
              },
              "radians": -0.0854565076452456,
              "degrees": -4.896297220000025,
              "degMinSec": {},
              "vrc": "E004.53.046.670",
              "nats": "0045347W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9578948710780435,
                "degrees": 54.88333333,
                "percentage": 142.1976644186302,
                "degMinSec": {}
              },
              "radians": 0.9578948710780435,
              "degrees": 54.88333333,
              "degMinSec": {},
              "vrc": "N054.52.060.000",
              "nats": "545300N"
            },
            "lon": {
              "angle": {
                "radians": -0.09003920906289586,
                "degrees": -5.158866670000005,
                "percentage": -9.028331842888736,
                "degMinSec": {}
              },
              "radians": -0.09003920906289586,
              "degrees": -5.158866670000005,
              "degMinSec": {},
              "vrc": "E005.09.031.920",
              "nats": "0050932W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": 11072.415501371579,
      "crossTrackDistance_m": 17.887591196034023,
      "requiredTrueCourse": 206.58506717658807,
      "turnRadius_m": 0
    },
    "autopilot": {
      "selectedHeading": 211,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 3.6826447217080354,
          "degrees": 211,
          "percentage": 60.086061902756036,
          "degMinSec": {}
        },
        "radians": 3.6826447217080354,
        "degrees": 211
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 290,
      "currentLateralMode": "LNAV",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "THRUST",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "CONNECTED",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "SHT4J",
    "delayMs": 59985,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9435652339393685,
          "degrees": 54.0623056,
          "percentage": 137.95341494663575,
          "degMinSec": {}
        },
        "radians": 0.9435652339393685,
        "degrees": 54.0623056,
        "degMinSec": {},
        "vrc": "N054.03.044.300",
        "nats": "540344N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07205204004698729,
          "degrees": -4.128277799999961,
          "percentage": -7.2176985485563145,
          "degMinSec": {}
        },
        "radians": -0.07205204004698729,
        "degrees": -4.128277799999961,
        "degMinSec": {},
        "vrc": "E004.07.041.800",
        "nats": "0040742W"
      },
      "indicatedAltitude": {
        "meters": 8090.563787902808,
        "feet": 26543.845297903048,
        "nauticalMiles": 4.368554961070631,
        "statuteMiles": 5.027243266761369
      },
      "trueAltitude": {
        "meters": 8229.599736652808,
        "feet": 27000,
        "nauticalMiles": 4.4436283675231145,
        "statuteMiles": 5.113636200000005
      },
      "pressureAltitude": {
        "meters": 8090.563787902808,
        "feet": 26543.845297903048,
        "nauticalMiles": 4.368554961070631,
        "statuteMiles": 5.027243266761369
      },
      "densityAltitude": {
        "meters": 8101.885812932536,
        "feet": 26580.991050501583,
        "nauticalMiles": 4.374668365514329,
        "statuteMiles": 5.034278446952632
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.387131650434046,
          "degrees": 308.6599072512164,
          "percentage": -124.99955724559403,
          "degMinSec": {}
        },
        "radians": 5.387131650434046,
        "degrees": 308.6599072512164
      },
      "track_True": {
        "angle": {
          "radians": 5.437463921960948,
          "degrees": 311.5437339830143,
          "percentage": -112.85574143856816,
          "degMinSec": {}
        },
        "radians": 5.437463921960948,
        "degrees": 311.5437339830143
      },
      "track_Mag": {
        "angle": {
          "radians": 5.449944644884358,
          "degrees": 312.2588267317979,
          "percentage": -110.05734279539926,
          "degMinSec": {}
        },
        "radians": 5.449944644884358,
        "degrees": 312.2588267317979
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 191.92671880718902,
        "knots": 373.07560079304153,
        "feetPerMinute": 37780.85136788268
      },
      "groundSpeed": {
        "metersPerSecond": 193.43975571751332,
        "knots": 376.0167085129539,
        "feetPerMinute": 38078.693288894785
      },
      "machNumber": 0.6156999421379458,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -97.13211174688453,
        "knots": -188.809672626511,
        "feetPerMinute": -19120.49505021892
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -167.28506197399085,
        "knots": -325.17606400777026,
        "feetPerMinute": -32930.13136360489
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.6699313334942834,
          "degrees": 210.27157651204064,
          "percentage": 58.3687531514251,
          "degMinSec": {}
        },
        "radians": 3.6699313334942834,
        "degrees": 210.27157651204064
      },
      "windSpeed": {
        "metersPerSecond": 10.371681538548607,
        "knots": 20.160930928618477,
        "feetPerMinute": 2041.6696595359087
      },
      "windXComp": {
        "metersPerSecond": 10.260725960918473,
        "knots": 19.945250594775608,
        "feetPerMinute": 2019.8280096971857
      },
      "windHComp": {
        "metersPerSecond": -1.5130369103242967,
        "knots": -2.941107719912422,
        "feetPerMinute": -297.8419210121019
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 8432.39453125,
          "feet": 27665.33727390625,
          "nauticalMiles": 4.553128796571274,
          "statuteMiles": 5.239647043298387
        },
        "levelPressure": {
          "pascals": 35000,
          "hectopascals": 350,
          "inchesOfMercury": 10.336680448907265
        },
        "temp": {
          "kelvin": 240.46995544433594,
          "celsius": -32.68004455566404
        },
        "v": {
          "metersPerSecond": 8.95745849609375,
          "knots": 17.411901952880857,
          "feetPerMinute": 1763.279287939453
        },
        "u": {
          "metersPerSecond": 5.228356838226318,
          "knots": 10.1631100698452,
          "feetPerMinute": 1029.204134947586
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9435652339393685,
            "degrees": 54.0623056,
            "percentage": 137.95341494663575,
            "degMinSec": {}
          },
          "radians": 0.9435652339393685,
          "degrees": 54.0623056,
          "degMinSec": {},
          "vrc": "N054.03.044.300",
          "nats": "540344N"
        },
        "lon": {
          "angle": {
            "radians": -0.07205204004698729,
            "degrees": -4.128277799999961,
            "percentage": -7.2176985485563145,
            "degMinSec": {}
          },
          "radians": -0.07205204004698729,
          "degrees": -4.128277799999961,
          "degMinSec": {},
          "vrc": "E004.07.041.800",
          "nats": "0040742W"
        },
        "alt": {
          "meters": 8229.599736652808,
          "feet": 27000,
          "nauticalMiles": 4.4436283675231145,
          "statuteMiles": 5.113636200000005
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9435688148208217,
                  "degrees": 54.0625107693942,
                  "percentage": 137.9544545226472,
                  "degMinSec": {}
                },
                "radians": 0.9435688148208217,
                "degrees": 54.0625107693942,
                "degMinSec": {},
                "vrc": "N054.03.045.039",
                "nats": "540345N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07204772449662222,
                  "degrees": -4.128030537177767,
                  "percentage": -7.217264745461527,
                  "degMinSec": {}
                },
                "radians": -0.07204772449662222,
                "degrees": -4.128030537177767,
                "degMinSec": {},
                "vrc": "E004.07.040.910",
                "nats": "0040741W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "percentage": 138.4284080202272,
                  "degMinSec": {}
                },
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "degMinSec": {},
                "vrc": "N054.09.021.020",
                "nats": "540921N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "percentage": -7.6142444922198695,
                  "degMinSec": {}
                },
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "degMinSec": {},
                "vrc": "E004.21.015.260",
                "nats": "0042115W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "MASOP"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.3272346202231535,
            "degrees": 305.22806021476464,
            "percentage": -141.61174763803515,
            "degMinSec": {}
          },
          "radians": 5.3272346202231535,
          "degrees": 305.22806021476464
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.324036137417867,
            "degrees": 305.0448006491766,
            "percentage": -142.57739316250385,
            "degMinSec": {}
          },
          "radians": 5.324036137417867,
          "degrees": 305.0448006491766
        },
        "legLength": {
          "meters": 18031.518139870772,
          "feet": 59158.525974013624,
          "nauticalMiles": 9.736240896258517,
          "statuteMiles": 11.204265924420616
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9435688148208217,
                  "degrees": 54.0625107693942,
                  "percentage": 137.9544545226472,
                  "degMinSec": {}
                },
                "radians": 0.9435688148208217,
                "degrees": 54.0625107693942,
                "degMinSec": {},
                "vrc": "N054.03.045.039",
                "nats": "540345N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07204772449662222,
                  "degrees": -4.128030537177767,
                  "percentage": -7.217264745461527,
                  "degMinSec": {}
                },
                "radians": -0.07204772449662222,
                "degrees": -4.128030537177767,
                "degMinSec": {},
                "vrc": "E004.07.040.910",
                "nats": "0040741W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "percentage": 138.4284080202272,
                  "degMinSec": {}
                },
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "degMinSec": {},
                "vrc": "N054.09.021.020",
                "nats": "540921N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "percentage": -7.6142444922198695,
                  "degMinSec": {}
                },
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "degMinSec": {},
                "vrc": "E004.21.015.260",
                "nats": "0042115W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.438593895987405,
              "degrees": 311.60847672568974,
              "percentage": -112.5991529192575,
              "degMinSec": {}
            },
            "radians": 5.438593895987405,
            "degrees": 311.60847672568974
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.420779718567154,
              "degrees": 310.5877995440121,
              "percentage": -116.72229413505232,
              "degMinSec": {}
            },
            "radians": 5.420779718567154,
            "degrees": 310.5877995440121
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAGEE"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9451976978100912,
                    "degrees": 54.15583889,
                    "percentage": 138.4284080202272,
                    "degMinSec": {}
                  },
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "degMinSec": {},
                  "vrc": "N054.09.021.020",
                  "nats": "540921N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07599580504888337,
                    "degrees": -4.35423889000001,
                    "percentage": -7.6142444922198695,
                    "degMinSec": {}
                  },
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "degMinSec": {},
                  "vrc": "E004.21.015.260",
                  "nats": "0042115W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MASOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 107526.38245727522,
            "feet": 352776.8566211268,
            "nauticalMiles": 58.05960175878791,
            "statuteMiles": 66.81379646444465
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9451976978100912,
                    "degrees": 54.15583889,
                    "percentage": 138.4284080202272,
                    "degMinSec": {}
                  },
                  "radians": 0.9451976978100912,
                  "degrees": 54.15583889,
                  "degMinSec": {},
                  "vrc": "N054.09.021.020",
                  "nats": "540921N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07599580504888337,
                    "degrees": -4.35423889000001,
                    "percentage": -7.6142444922198695,
                    "degMinSec": {}
                  },
                  "radians": -0.07599580504888337,
                  "degrees": -4.35423889000001,
                  "degMinSec": {},
                  "vrc": "E004.21.015.260",
                  "nats": "0042115W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> MASOP; MASOP =(TF)=> MAGEE; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9435688148208217,
                "degrees": 54.0625107693942,
                "percentage": 137.9544545226472,
                "degMinSec": {}
              },
              "radians": 0.9435688148208217,
              "degrees": 54.0625107693942,
              "degMinSec": {},
              "vrc": "N054.03.045.039",
              "nats": "540345N"
            },
            "lon": {
              "angle": {
                "radians": -0.07204772449662222,
                "degrees": -4.128030537177767,
                "percentage": -7.217264745461527,
                "degMinSec": {}
              },
              "radians": -0.07204772449662222,
              "degrees": -4.128030537177767,
              "degMinSec": {},
              "vrc": "E004.07.040.910",
              "nats": "0040741W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "percentage": 138.4284080202272,
                "degMinSec": {}
              },
              "radians": 0.9451976978100912,
              "degrees": 54.15583889,
              "degMinSec": {},
              "vrc": "N054.09.021.020",
              "nats": "540921N"
            },
            "lon": {
              "angle": {
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "percentage": -7.6142444922198695,
                "degMinSec": {}
              },
              "radians": -0.07599580504888337,
              "degrees": -4.35423889000001,
              "degMinSec": {},
              "vrc": "E004.21.015.260",
              "nats": "0042115W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9451976978100912,
                "degrees": 54.15583889,
                "percentage": 138.4284080202272,
                "degMinSec": {}
              },
              "radians": 0.9451976978100912,
              "degrees": 54.15583889,
              "degMinSec": {},
              "vrc": "N054.09.021.020",
              "nats": "540921N"
            },
            "lon": {
              "angle": {
                "radians": -0.07599580504888337,
                "degrees": -4.35423889000001,
                "percentage": -7.6142444922198695,
                "degMinSec": {}
              },
              "radians": -0.07599580504888337,
              "degrees": -4.35423889000001,
              "degMinSec": {},
              "vrc": "E004.21.015.260",
              "nats": "0042115W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY687T",
    "delayMs": 61038,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9423788935042152,
          "degrees": 53.9943333,
          "percentage": 137.60956931379638,
          "degMinSec": {}
        },
        "radians": 0.9423788935042152,
        "degrees": 53.9943333,
        "degMinSec": {},
        "vrc": "N053.59.039.600",
        "nats": "535940N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07346527153957183,
          "degrees": -4.209250000000029,
          "percentage": -7.359772509721122,
          "degMinSec": {}
        },
        "radians": -0.07346527153957183,
        "degrees": -4.209250000000029,
        "degMinSec": {},
        "vrc": "E004.12.033.300",
        "nats": "0041233W"
      },
      "indicatedAltitude": {
        "meters": 7176.163817163608,
        "feet": 23543.84529790305,
        "nauticalMiles": 3.874818475790285,
        "statuteMiles": 4.459061466761368
      },
      "trueAltitude": {
        "meters": 7315.199765913608,
        "feet": 24000,
        "nauticalMiles": 3.9498918822427687,
        "statuteMiles": 4.545454400000005
      },
      "pressureAltitude": {
        "meters": 7176.163817163608,
        "feet": 23543.84529790305,
        "nauticalMiles": 3.874818475790285,
        "statuteMiles": 4.459061466761368
      },
      "densityAltitude": {
        "meters": 7266.77955812549,
        "feet": 23841.141045480435,
        "nauticalMiles": 3.923747061622835,
        "statuteMiles": 4.515367477758323
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.386703218873489,
          "degrees": 308.6353599309862,
          "percentage": -125.10940119104139,
          "degMinSec": {}
        },
        "radians": 5.386703218873489,
        "degrees": 308.6353599309862
      },
      "track_True": {
        "angle": {
          "radians": 5.43118978187351,
          "degrees": 311.1842522359303,
          "percentage": -114.29244795736697,
          "degMinSec": {}
        },
        "radians": 5.43118978187351,
        "degrees": 311.1842522359303
      },
      "track_Mag": {
        "angle": {
          "radians": 5.444098936357477,
          "degrees": 311.923892304944,
          "percentage": -111.35836716347079,
          "degMinSec": {}
        },
        "radians": 5.444098936357477,
        "degrees": 311.923892304944
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 183.78161966303986,
        "knots": 357.24279869228207,
        "feetPerMinute": 36177.48534331726
      },
      "groundSpeed": {
        "metersPerSecond": 182.71730005915623,
        "knots": 355.17392741619045,
        "feetPerMinute": 35967.97360356493
      },
      "machNumber": 0.5807701735574268,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -30.29639268073222,
        "knots": -58.89146113408524,
        "feetPerMinute": -5963.85701775921
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -180.18806933713066,
        "knots": -350.25749745256536,
        "feetPerMinute": -35470.0935242419
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.939898830195596,
          "degrees": 225.73957467873785,
          "percentage": 102.61550892967574,
          "degMinSec": {}
        },
        "radians": 3.939898830195596,
        "degrees": 225.73957467873785
      },
      "windSpeed": {
        "metersPerSecond": 8.605814710605769,
        "knots": 16.72836129032276,
        "feetPerMinute": 1694.0580681086299
      },
      "windXComp": {
        "metersPerSecond": 8.539746519310022,
        "knots": 16.599935033081668,
        "feetPerMinute": 1681.0525182247854
      },
      "windHComp": {
        "metersPerSecond": 1.0643196038836353,
        "knots": 2.0688712760915813,
        "feetPerMinute": 209.51173975233516
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 7477.01806640625,
          "feet": 24530.899952988282,
          "nauticalMiles": 4.037266774517414,
          "statuteMiles": 4.646003630302937
        },
        "levelPressure": {
          "pascals": 40000,
          "hectopascals": 400,
          "inchesOfMercury": 11.813349084465447
        },
        "temp": {
          "kelvin": 248.119384765625,
          "celsius": -25.030615234374977
        },
        "v": {
          "metersPerSecond": 6.006176948547363,
          "knots": 11.6750710243721,
          "feetPerMinute": 1182.318334792328
        },
        "u": {
          "metersPerSecond": 6.16326904296875,
          "knots": 11.980433549560546,
          "feetPerMinute": 1213.2419764160156
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9423788935042152,
            "degrees": 53.9943333,
            "percentage": 137.60956931379638,
            "degMinSec": {}
          },
          "radians": 0.9423788935042152,
          "degrees": 53.9943333,
          "degMinSec": {},
          "vrc": "N053.59.039.600",
          "nats": "535940N"
        },
        "lon": {
          "angle": {
            "radians": -0.07346527153957183,
            "degrees": -4.209250000000029,
            "percentage": -7.359772509721122,
            "degMinSec": {}
          },
          "radians": -0.07346527153957183,
          "degrees": -4.209250000000029,
          "degMinSec": {},
          "vrc": "E004.12.033.300",
          "nats": "0041233W"
        },
        "alt": {
          "meters": 7315.199765913608,
          "feet": 24000,
          "nauticalMiles": 3.9498918822427687,
          "statuteMiles": 4.545454400000005
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9423820252626474,
                  "degrees": 53.99451273654062,
                  "percentage": 137.61047553564734,
                  "degMinSec": {}
                },
                "radians": 0.9423820252626474,
                "degrees": 53.99451273654062,
                "degMinSec": {},
                "vrc": "N053.59.040.246",
                "nats": "535940N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07346152018566698,
                  "degrees": -4.209035063253822,
                  "percentage": -7.35939534246698,
                  "degMinSec": {}
                },
                "radians": -0.07346152018566698,
                "degrees": -4.209035063253822,
                "degMinSec": {},
                "vrc": "E004.12.032.526",
                "nats": "0041233W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "percentage": 138.0824602372339,
                  "degMinSec": {}
                },
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "degMinSec": {},
                "vrc": "N054.05.015.930",
                "nats": "540516N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "percentage": -7.75714864160994,
                  "degMinSec": {}
                },
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "degMinSec": {},
                "vrc": "E004.26.008.290",
                "nats": "0042608W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "NOPKI"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.325233353332428,
            "degrees": 305.1133960682468,
            "percentage": -142.21491633215177,
            "degMinSec": {}
          },
          "radians": 5.325233353332428,
          "degrees": 305.1133960682468
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.3220320735693,
            "degrees": 304.9299762487789,
            "percentage": -143.18693440385212,
            "degMinSec": {}
          },
          "radians": 5.3220320735693,
          "degrees": 304.9299762487789
        },
        "legLength": {
          "meters": 18067.02207589412,
          "feet": 59275.00870747647,
          "nauticalMiles": 9.75541148806378,
          "statuteMiles": 11.226327047476563
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9423820252626474,
                  "degrees": 53.99451273654062,
                  "percentage": 137.61047553564734,
                  "degMinSec": {}
                },
                "radians": 0.9423820252626474,
                "degrees": 53.99451273654062,
                "degMinSec": {},
                "vrc": "N053.59.040.246",
                "nats": "535940N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07346152018566698,
                  "degrees": -4.209035063253822,
                  "percentage": -7.35939534246698,
                  "degMinSec": {}
                },
                "radians": -0.07346152018566698,
                "degrees": -4.209035063253822,
                "degMinSec": {},
                "vrc": "E004.12.032.526",
                "nats": "0041233W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "percentage": 138.0824602372339,
                  "degMinSec": {}
                },
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "degMinSec": {},
                "vrc": "N054.05.015.930",
                "nats": "540516N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "percentage": -7.75714864160994,
                  "degMinSec": {}
                },
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "degMinSec": {},
                "vrc": "E004.26.008.290",
                "nats": "0042608W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.3257006040392305,
              "degrees": 305.1401675617211,
              "percentage": -142.07378319360646,
              "degMinSec": {}
            },
            "radians": 5.3257006040392305,
            "degrees": 305.1401675617211
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.312960633207489,
              "degrees": 304.4102210019425,
              "percentage": -145.99044233056168,
              "degMinSec": {}
            },
            "radians": 5.312960633207489,
            "degrees": 304.4102210019425
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9440094678814897,
                    "degrees": 54.08775833,
                    "percentage": 138.0824602372339,
                    "degMinSec": {}
                  },
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "degMinSec": {},
                  "vrc": "N054.05.015.930",
                  "nats": "540516N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07741645453985413,
                    "degrees": -4.435636110000043,
                    "percentage": -7.75714864160994,
                    "degMinSec": {}
                  },
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "degMinSec": {},
                  "vrc": "E004.26.008.290",
                  "nats": "0042608W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "NOPKI"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 71082.94499483609,
            "feet": 233211.769256858,
            "nauticalMiles": 38.38171975963071,
            "statuteMiles": 44.168894279182126
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9440094678814897,
                    "degrees": 54.08775833,
                    "percentage": 138.0824602372339,
                    "degMinSec": {}
                  },
                  "radians": 0.9440094678814897,
                  "degrees": 54.08775833,
                  "degMinSec": {},
                  "vrc": "N054.05.015.930",
                  "nats": "540516N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07741645453985413,
                    "degrees": -4.435636110000043,
                    "percentage": -7.75714864160994,
                    "degMinSec": {}
                  },
                  "radians": -0.07741645453985413,
                  "degrees": -4.435636110000043,
                  "degMinSec": {},
                  "vrc": "E004.26.008.290",
                  "nats": "0042608W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.480023517854051,
              "degrees": 313.9822192054715,
              "percentage": -103.617363161653,
              "degMinSec": {}
            },
            "radians": 5.480023517854051,
            "degrees": 313.9822192054715
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.478352016380683,
              "degrees": 313.8864492255976,
              "percentage": -103.96457671864067,
              "degMinSec": {}
            },
            "radians": 5.478352016380683,
            "degrees": 313.8864492255976
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 10553.672770561494,
            "feet": 34624.91177256897,
            "nauticalMiles": 5.698527413910094,
            "statuteMiles": 6.557748231926483
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.1474641895474593,
              "degrees": 180.3364142296336,
              "percentage": 0.5871603432202414,
              "degMinSec": {}
            },
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 6.165737882555982,
              "degrees": 353.27075825438664,
              "percentage": -11.799044018857247,
              "degMinSec": {}
            },
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16253794.83622146,
            "feet": 53326100.25046881,
            "nauticalMiles": 8776.347103791286,
            "statuteMiles": 10099.63987576395
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> NOPKI; NOPKI =(TF)=> ROBOP; ROBOP =(TF)=> IPSET; IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9423820252626474,
                "degrees": 53.99451273654062,
                "percentage": 137.61047553564734,
                "degMinSec": {}
              },
              "radians": 0.9423820252626474,
              "degrees": 53.99451273654062,
              "degMinSec": {},
              "vrc": "N053.59.040.246",
              "nats": "535940N"
            },
            "lon": {
              "angle": {
                "radians": -0.07346152018566698,
                "degrees": -4.209035063253822,
                "percentage": -7.35939534246698,
                "degMinSec": {}
              },
              "radians": -0.07346152018566698,
              "degrees": -4.209035063253822,
              "degMinSec": {},
              "vrc": "E004.12.032.526",
              "nats": "0041233W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "percentage": 138.0824602372339,
                "degMinSec": {}
              },
              "radians": 0.9440094678814897,
              "degrees": 54.08775833,
              "degMinSec": {},
              "vrc": "N054.05.015.930",
              "nats": "540516N"
            },
            "lon": {
              "angle": {
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "percentage": -7.75714864160994,
                "degMinSec": {}
              },
              "radians": -0.07741645453985413,
              "degrees": -4.435636110000043,
              "degMinSec": {},
              "vrc": "E004.26.008.290",
              "nats": "0042608W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9440094678814897,
                "degrees": 54.08775833,
                "percentage": 138.0824602372339,
                "degMinSec": {}
              },
              "radians": 0.9440094678814897,
              "degrees": 54.08775833,
              "degMinSec": {},
              "vrc": "N054.05.015.930",
              "nats": "540516N"
            },
            "lon": {
              "angle": {
                "radians": -0.07741645453985413,
                "degrees": -4.435636110000043,
                "percentage": -7.75714864160994,
                "degMinSec": {}
              },
              "radians": -0.07741645453985413,
              "degrees": -4.435636110000043,
              "degMinSec": {},
              "vrc": "E004.26.008.290",
              "nats": "0042608W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "AWC6977",
    "delayMs": 301425,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9540652464204387,
          "degrees": 54.66391200000001,
          "percentage": 141.04660772548115,
          "degMinSec": {}
        },
        "radians": 0.9540652464204387,
        "degrees": 54.66391200000001,
        "degMinSec": {},
        "vrc": "N054.39.050.083",
        "nats": "543950N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10818829332070834,
          "degrees": -6.198732599999982,
          "percentage": -10.861238305442175,
          "degMinSec": {}
        },
        "radians": -0.10818829332070834,
        "degrees": -6.198732599999982,
        "degMinSec": {},
        "vrc": "E006.11.055.437",
        "nats": "0061155W"
      },
      "indicatedAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "trueAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "pressureAltitude": {
        "meters": 67.88681551103528,
        "feet": 222.72577980122497,
        "nauticalMiles": 0.036655947900127044,
        "statuteMiles": 0.04218291149128793
      },
      "densityAltitude": {
        "meters": -72.58308962302895,
        "feet": -238.1335037588183,
        "nauticalMiles": -0.039191733057791014,
        "statuteMiles": -0.045101040935330766
      },
      "heading_Mag": {
        "angle": {
          "radians": 1.325359400733194,
          "degrees": 75.9375,
          "percentage": 399.2223783770083,
          "degMinSec": {}
        },
        "radians": 1.325359400733194,
        "degrees": 75.9375
      },
      "heading_True": {
        "angle": {
          "radians": 1.297601920370444,
          "degrees": 74.34711352529717,
          "percentage": 356.88758960404806,
          "degMinSec": {}
        },
        "radians": 1.297601920370444,
        "degrees": 74.34711352529717
      },
      "track_True": {
        "angle": {
          "radians": 1.297601920370444,
          "degrees": 74.34711352529717,
          "percentage": 356.88758960404806,
          "degMinSec": {}
        },
        "radians": 1.297601920370444,
        "degrees": 74.34711352529717
      },
      "track_Mag": {
        "angle": {
          "radians": 1.325359400733194,
          "degrees": 75.9375,
          "percentage": 399.2223783770083,
          "degMinSec": {}
        },
        "radians": 1.325359400733194,
        "degrees": 75.9375
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "trueAirSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "groundSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "machNumber": 0,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -0,
        "knots": -0,
        "feetPerMinute": -0
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101475.9140625,
        "hectopascals": 1014.759140625,
        "inchesOfMercury": 29.969259912138217
      },
      "surfacePressure": {
        "pascals": 101475.9140625,
        "hectopascals": 1014.759140625,
        "inchesOfMercury": 29.969259912138217
      },
      "windDirection": {
        "angle": {
          "radians": 2.5068660660370234,
          "degrees": 143.63284538848538,
          "percentage": -73.63790986581812,
          "degMinSec": {}
        },
        "radians": 2.5068660660370234,
        "degrees": 143.63284538848538
      },
      "windSpeed": {
        "metersPerSecond": 6.570514368371236,
        "knots": 12.772054931872216,
        "feetPerMinute": 1293.408381619625
      },
      "windXComp": {
        "metersPerSecond": -6.145769889122522,
        "knots": -11.94641792435148,
        "feetPerMinute": -1209.797260981724
      },
      "windHComp": {
        "metersPerSecond": 2.3240420682353853,
        "knots": 4.517575230086944,
        "feetPerMinute": 457.48861074896286
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9555677654668955,
            "degrees": 54.75000000000001,
            "percentage": 141.49672721156952,
            "degMinSec": {}
          },
          "radians": 0.9555677654668955,
          "degrees": 54.75000000000001,
          "degMinSec": {},
          "vrc": "N054.45.000.000",
          "nats": "544500N"
        },
        "lon": {
          "angle": {
            "radians": -0.10908254561742758,
            "degrees": -6.249969482421875,
            "percentage": -10.95172726625836,
            "degMinSec": {}
          },
          "radians": -0.10908254561742758,
          "degrees": -6.249969482421875,
          "degMinSec": {},
          "vrc": "E006.14.059.890",
          "nats": "0061500W"
        },
        "geoPotentialHeight": {
          "meters": 212.58175659179688,
          "feet": 697.4467302966309,
          "nauticalMiles": 0.11478496576230933,
          "statuteMiles": 0.13209217954135155
        },
        "levelPressure": {
          "pascals": 100000,
          "hectopascals": 1000,
          "inchesOfMercury": 29.533372711163615
        },
        "temp": {
          "kelvin": 285.9950866699219,
          "celsius": 12.845086669921898
        },
        "v": {
          "metersPerSecond": 5.29080057144165,
          "knots": 10.284490945993422,
          "feetPerMinute": 1041.4962088085174
        },
        "u": {
          "metersPerSecond": -3.8960349559783936,
          "knots": -7.573284172968864,
          "feetPerMinute": -766.9360394983291
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 101475.9140625,
          "hectopascals": 1014.759140625,
          "inchesOfMercury": 29.969259912138217
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9540652464204387,
            "degrees": 54.66391200000001,
            "percentage": 141.04660772548115,
            "degMinSec": {}
          },
          "radians": 0.9540652464204387,
          "degrees": 54.66391200000001,
          "degMinSec": {},
          "vrc": "N054.39.050.083",
          "nats": "543950N"
        },
        "lon": {
          "angle": {
            "radians": -0.10818829332070834,
            "degrees": -6.198732599999982,
            "percentage": -10.861238305442175,
            "degMinSec": {}
          },
          "radians": -0.10818829332070834,
          "degrees": -6.198732599999982,
          "degMinSec": {},
          "vrc": "E006.11.055.437",
          "nats": "0061155W"
        },
        "alt": {
          "meters": 81.68639738603528,
          "feet": 268,
          "nauticalMiles": 0.04410712601837758,
          "statuteMiles": 0.050757574133333386
        }
      },
      "onGround": true,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9540653041592922,
                  "degrees": 54.663915308192635,
                  "percentage": 141.04662498601948,
                  "degMinSec": {}
                },
                "radians": 0.9540653041592922,
                "degrees": 54.663915308192635,
                "degMinSec": {},
                "vrc": "N054.39.050.095",
                "nats": "543950N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10818818711309675,
                  "degrees": -6.198726514752085,
                  "percentage": -10.86122755939174,
                  "degMinSec": {}
                },
                "radians": -0.10818818711309675,
                "degrees": -6.198726514752085,
                "degMinSec": {},
                "vrc": "E006.11.055.415",
                "nats": "0061155W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPOD"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 2.387142578610529,
            "degrees": 136.77319485035966,
            "percentage": -93.99432885655035,
            "degMinSec": {}
          },
          "radians": 2.387142578610529,
          "degrees": 136.77319485035966
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 2.3960834347218434,
            "degrees": 137.2854681707717,
            "percentage": -92.32431465031617,
            "degMinSec": {}
          },
          "radians": 2.3960834347218434,
          "degrees": 137.2854681707717
        },
        "legLength": {
          "meters": 59679.52802575767,
          "feet": 195798.98272802678,
          "nauticalMiles": 32.22436718453438,
          "statuteMiles": 37.08313948152642
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9540653041592922,
                  "degrees": 54.663915308192635,
                  "percentage": 141.04662498601948,
                  "degMinSec": {}
                },
                "radians": 0.9540653041592922,
                "degrees": 54.663915308192635,
                "degMinSec": {},
                "vrc": "N054.39.050.095",
                "nats": "543950N"
              },
              "lon": {
                "angle": {
                  "radians": -0.10818818711309675,
                  "degrees": -6.198726514752085,
                  "percentage": -10.86122755939174,
                  "degMinSec": {}
                },
                "radians": -0.10818818711309675,
                "degrees": -6.198726514752085,
                "degMinSec": {},
                "vrc": "E006.11.055.415",
                "nats": "0061155W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.194801594813155,
              "degrees": 125.75286825137599,
              "percentage": -138.89409825386858,
              "degMinSec": {}
            },
            "radians": 2.194801594813155,
            "degrees": 125.75286825137599
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.204627895223652,
              "degrees": 126.31587379312512,
              "percentage": -136.05448462062563,
              "degMinSec": {}
            },
            "radians": 2.204627895223652,
            "degrees": 126.31587379312512
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAKUX"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPOD"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 55991.55979285222,
            "feet": 183699.34903078128,
            "nauticalMiles": 30.233023646248498,
            "statuteMiles": 34.79154226371256
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9420409789114731,
                    "degrees": 53.97497222,
                    "percentage": 137.51183446058306,
                    "degMinSec": {}
                  },
                  "radians": 0.9420409789114731,
                  "degrees": 53.97497222,
                  "degMinSec": {},
                  "vrc": "N053.58.029.900",
                  "nats": "535830N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0850745714854444,
                    "degrees": -4.874413889999983,
                    "percentage": -8.528041502320681,
                    "degMinSec": {}
                  },
                  "radians": -0.0850745714854444,
                  "degrees": -4.874413889999983,
                  "degMinSec": {},
                  "vrc": "E004.52.027.890",
                  "nats": "0045228W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPOD; PEPOD =(TF)=> MAKUX; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540653041592922,
                "degrees": 54.663915308192635,
                "percentage": 141.04662498601948,
                "degMinSec": {}
              },
              "radians": 0.9540653041592922,
              "degrees": 54.663915308192635,
              "degMinSec": {},
              "vrc": "N054.39.050.095",
              "nats": "543950N"
            },
            "lon": {
              "angle": {
                "radians": -0.10818818711309675,
                "degrees": -6.198726514752085,
                "percentage": -10.86122755939174,
                "degMinSec": {}
              },
              "radians": -0.10818818711309675,
              "degrees": -6.198726514752085,
              "degMinSec": {},
              "vrc": "E006.11.055.415",
              "nats": "0061155W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9420409789114731,
                "degrees": 53.97497222,
                "percentage": 137.51183446058306,
                "degMinSec": {}
              },
              "radians": 0.9420409789114731,
              "degrees": 53.97497222,
              "degMinSec": {},
              "vrc": "N053.58.029.900",
              "nats": "535830N"
            },
            "lon": {
              "angle": {
                "radians": -0.0850745714854444,
                "degrees": -4.874413889999983,
                "percentage": -8.528041502320681,
                "degMinSec": {}
              },
              "radians": -0.0850745714854444,
              "degrees": -4.874413889999983,
              "degMinSec": {},
              "vrc": "E004.52.027.890",
              "nats": "0045228W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 76,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 1.3264502315156905,
          "degrees": 76,
          "percentage": 401.07809335358456,
          "degMinSec": {}
        },
        "radians": 1.3264502315156905,
        "degrees": 76
      },
      "selectedAltitude": 5000,
      "selectedAltitudeLength": {
        "meters": 1523.9999512320016,
        "feet": 5000,
        "nauticalMiles": 0.8228941421339102,
        "statuteMiles": 0.9469696666666677
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "ON_GROUND"
  },
  {
    "callsign": "EZY857W",
    "delayMs": 361940,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "percentage": 137.3334254963888,
          "degMinSec": {}
        },
        "radians": 0.9414233257387483,
        "degrees": 53.9395833,
        "degMinSec": {},
        "vrc": "N053.56.022.500",
        "nats": "535622N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "percentage": -7.561162063112771,
          "degMinSec": {}
        },
        "radians": -0.07546801940294223,
        "degrees": -4.323998999999997,
        "degMinSec": {},
        "vrc": "E004.19.026.396",
        "nats": "0041926W"
      },
      "indicatedAltitude": {
        "meters": 7176.163817163608,
        "feet": 23543.84529790305,
        "nauticalMiles": 3.874818475790285,
        "statuteMiles": 4.459061466761368
      },
      "trueAltitude": {
        "meters": 7315.199765913608,
        "feet": 24000,
        "nauticalMiles": 3.9498918822427687,
        "statuteMiles": 4.545454400000005
      },
      "pressureAltitude": {
        "meters": 7176.163817163608,
        "feet": 23543.84529790305,
        "nauticalMiles": 3.874818475790285,
        "statuteMiles": 4.459061466761368
      },
      "densityAltitude": {
        "meters": 7266.77955812549,
        "feet": 23841.141045480435,
        "nauticalMiles": 3.923747061622835,
        "statuteMiles": 4.515367477758323
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.386033308863389,
          "degrees": 308.59697691475395,
          "percentage": -125.28139310935545,
          "degMinSec": {}
        },
        "radians": 5.386033308863389,
        "degrees": 308.59697691475395
      },
      "track_True": {
        "angle": {
          "radians": 5.430516152531586,
          "degrees": 311.1456561176819,
          "percentage": -114.44792522706766,
          "degMinSec": {}
        },
        "radians": 5.430516152531586,
        "degrees": 311.1456561176819
      },
      "track_Mag": {
        "angle": {
          "radians": 5.444095217025655,
          "degrees": 311.923679202928,
          "percentage": -111.35920032276199,
          "degMinSec": {}
        },
        "radians": 5.444095217025655,
        "degrees": 311.923679202928
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 183.78161966303986,
        "knots": 357.24279869228207,
        "feetPerMinute": 36177.48534331726
      },
      "groundSpeed": {
        "metersPerSecond": 182.7115794367295,
        "knots": 355.16280741861,
        "feetPerMinute": 35966.84749675197
      },
      "machNumber": 0.5807701735574268,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -23.320266162493226,
        "knots": -45.33095945836548,
        "feetPerMinute": -4590.6037221932565
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -181.21723551134642,
        "knots": -352.25803594531766,
        "feetPerMinute": -35672.68529730275
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.939898830195596,
          "degrees": 225.73957467873785,
          "percentage": 102.61550892967574,
          "degMinSec": {}
        },
        "radians": 3.939898830195596,
        "degrees": 225.73957467873785
      },
      "windSpeed": {
        "metersPerSecond": 8.605814710605769,
        "knots": 16.72836129032276,
        "feetPerMinute": 1694.0580681086299
      },
      "windXComp": {
        "metersPerSecond": 8.539031604775584,
        "knots": 16.59854535075339,
        "feetPerMinute": 1680.9117870127156
      },
      "windHComp": {
        "metersPerSecond": 1.0700402263103737,
        "knots": 2.079991273672062,
        "feetPerMinute": 210.63784656528756
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 7477.01806640625,
          "feet": 24530.899952988282,
          "nauticalMiles": 4.037266774517414,
          "statuteMiles": 4.646003630302937
        },
        "levelPressure": {
          "pascals": 40000,
          "hectopascals": 400,
          "inchesOfMercury": 11.813349084465447
        },
        "temp": {
          "kelvin": 248.119384765625,
          "celsius": -25.030615234374977
        },
        "v": {
          "metersPerSecond": 6.006176948547363,
          "knots": 11.6750710243721,
          "feetPerMinute": 1182.318334792328
        },
        "u": {
          "metersPerSecond": 6.16326904296875,
          "knots": 11.980433549560546,
          "feetPerMinute": 1213.2419764160156
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9414233257387483,
            "degrees": 53.9395833,
            "percentage": 137.3334254963888,
            "degMinSec": {}
          },
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "degMinSec": {},
          "vrc": "N053.56.022.500",
          "nats": "535622N"
        },
        "lon": {
          "angle": {
            "radians": -0.07546801940294223,
            "degrees": -4.323998999999997,
            "percentage": -7.561162063112771,
            "degMinSec": {}
          },
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "degMinSec": {},
          "vrc": "E004.19.026.396",
          "nats": "0041926W"
        },
        "alt": {
          "meters": 7315.199765913608,
          "feet": 24000,
          "nauticalMiles": 3.9498918822427687,
          "statuteMiles": 4.545454400000005
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9414270215944395,
                  "degrees": 53.939795056932795,
                  "percentage": 137.3344921431168,
                  "degMinSec": {}
                },
                "radians": 0.9414270215944395,
                "degrees": 53.939795056932795,
                "degMinSec": {},
                "vrc": "N053.56.023.262",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546367701677958,
                  "degrees": -4.323750199599861,
                  "percentage": -7.560725342046842,
                  "degMinSec": {}
                },
                "radians": -0.07546367701677958,
                "degrees": -4.323750199599861,
                "degMinSec": {},
                "vrc": "E004.19.025.501",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPEG"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.316535270583688,
            "degrees": 304.61503263688843,
            "percentage": -144.87691499202552,
            "degMinSec": {}
          },
          "radians": 5.316535270583688,
          "degrees": 304.61503263688843
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.313791225213212,
            "degrees": 304.4578104183677,
            "percentage": -145.73067225132996,
            "degMinSec": {}
          },
          "radians": 5.313791225213212,
          "degrees": 304.4578104183677
        },
        "legLength": {
          "meters": 15430.795914971615,
          "feet": 50625.97246967547,
          "nauticalMiles": 8.331963237025711,
          "statuteMiles": 9.588252054856895
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9414270215944395,
                  "degrees": 53.939795056932795,
                  "percentage": 137.3344921431168,
                  "degMinSec": {}
                },
                "radians": 0.9414270215944395,
                "degrees": 53.939795056932795,
                "degMinSec": {},
                "vrc": "N053.56.023.262",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546367701677958,
                  "degrees": -4.323750199599861,
                  "percentage": -7.560725342046842,
                  "degMinSec": {}
                },
                "radians": -0.07546367701677958,
                "degrees": -4.323750199599861,
                "degMinSec": {},
                "vrc": "E004.19.025.501",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.460531278681644,
              "degrees": 312.8653961676329,
              "percentage": -107.74323687641969,
              "degMinSec": {}
            },
            "radians": 5.460531278681644,
            "degrees": 312.8653961676329
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.447293429813374,
              "degrees": 312.1069232976491,
              "percentage": -110.64531286515107,
              "degMinSec": {}
            },
            "radians": 5.447293429813374,
            "degrees": 312.1069232976491
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPEG"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 82274.60605599961,
            "feet": 269929.8185327658,
            "nauticalMiles": 44.424733291576466,
            "statuteMiles": 51.123070055873455
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.1474641895474593,
              "degrees": 180.3364142296336,
              "percentage": 0.5871603432202414,
              "degMinSec": {}
            },
            "radians": 3.1474641895474593,
            "degrees": 180.3364142296336
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 6.165737882555982,
              "degrees": 353.27075825438664,
              "percentage": -11.799044018857247,
              "degMinSec": {}
            },
            "radians": 6.165737882555982,
            "degrees": 353.27075825438664
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "IPSET"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16253794.83622146,
            "feet": 53326100.25046881,
            "nauticalMiles": 8776.347103791286,
            "statuteMiles": 10099.63987576395
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.951522625633392,
                    "degrees": 54.51823056,
                    "percentage": 140.28922751224925,
                    "degMinSec": {}
                  },
                  "radians": 0.951522625633392,
                  "degrees": 54.51823056,
                  "degMinSec": {},
                  "vrc": "N054.31.005.630",
                  "nats": "543106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09516368965283029,
                    "degrees": -5.452477779999958,
                    "percentage": -9.545200562109763,
                    "degMinSec": {}
                  },
                  "radians": -0.09516368965283029,
                  "degrees": -5.452477779999958,
                  "degMinSec": {},
                  "vrc": "E005.27.008.920",
                  "nats": "0052709W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 3.0099666824725304,
              "degrees": 172.45838738066996,
              "percentage": -13.239143370208895,
              "degMinSec": {}
            },
            "radians": 3.0099666824725304,
            "degrees": 172.45838738066996
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 0.0065996052660315385,
              "degrees": 0.37812952819592005,
              "percentage": 0.6599701082507262,
              "degMinSec": {}
            },
            "radians": 0.0065996052660315385,
            "degrees": 0.37812952819592005
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "BELZU"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "P6"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 16269369.776435247,
            "feet": 53377199.13731981,
            "nauticalMiles": 8784.756898723135,
            "statuteMiles": 10109.317694933616
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": -1.5417075058701468,
                    "degrees": -88.33333333,
                    "percentage": -3436.7770818435856,
                    "degMinSec": {}
                  },
                  "radians": -1.5417075058701468,
                  "degrees": -88.33333333,
                  "degMinSec": {},
                  "vrc": "N088.19.060.000",
                  "nats": "882000S"
                },
                "lon": {
                  "angle": {
                    "radians": -3.12413936106985,
                    "degrees": -179,
                    "percentage": 1.745506492821751,
                    "degMinSec": {}
                  },
                  "radians": -3.12413936106985,
                  "degrees": -179,
                  "degMinSec": {},
                  "vrc": "E179.00.000.000",
                  "nats": "1790000W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9540163616679523,
                    "degrees": 54.66111111,
                    "percentage": 141.0319950539569,
                    "degMinSec": {}
                  },
                  "radians": 0.9540163616679523,
                  "degrees": 54.66111111,
                  "degMinSec": {},
                  "vrc": "N054.39.040.000",
                  "nats": "543940N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.10873401239924618,
                    "degrees": -6.2299999999999685,
                    "percentage": -10.916457257390238,
                    "degMinSec": {}
                  },
                  "radians": -0.10873401239924618,
                  "degrees": -6.2299999999999685,
                  "degMinSec": {},
                  "vrc": "E006.13.048.000",
                  "nats": "0061348W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPEG; PEPEG =(TF)=> IPSET; IPSET =(TF)=> P6; P6 =(TF)=> BELZU; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9414270215944395,
                "degrees": 53.939795056932795,
                "percentage": 137.3344921431168,
                "degMinSec": {}
              },
              "radians": 0.9414270215944395,
              "degrees": 53.939795056932795,
              "degMinSec": {},
              "vrc": "N053.56.023.262",
              "nats": "535623N"
            },
            "lon": {
              "angle": {
                "radians": -0.07546367701677958,
                "degrees": -4.323750199599861,
                "percentage": -7.560725342046842,
                "degMinSec": {}
              },
              "radians": -0.07546367701677958,
              "degrees": -4.323750199599861,
              "degMinSec": {},
              "vrc": "E004.19.025.501",
              "nats": "0041926W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.951522625633392,
                "degrees": 54.51823056,
                "percentage": 140.28922751224925,
                "degMinSec": {}
              },
              "radians": 0.951522625633392,
              "degrees": 54.51823056,
              "degMinSec": {},
              "vrc": "N054.31.005.630",
              "nats": "543106N"
            },
            "lon": {
              "angle": {
                "radians": -0.09516368965283029,
                "degrees": -5.452477779999958,
                "percentage": -9.545200562109763,
                "degMinSec": {}
              },
              "radians": -0.09516368965283029,
              "degrees": -5.452477779999958,
              "degMinSec": {},
              "vrc": "E005.27.008.920",
              "nats": "0052709W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": -1.5417075058701468,
                "degrees": -88.33333333,
                "percentage": -3436.7770818435856,
                "degMinSec": {}
              },
              "radians": -1.5417075058701468,
              "degrees": -88.33333333,
              "degMinSec": {},
              "vrc": "N088.19.060.000",
              "nats": "882000S"
            },
            "lon": {
              "angle": {
                "radians": -3.12413936106985,
                "degrees": -179,
                "percentage": 1.745506492821751,
                "degMinSec": {}
              },
              "radians": -3.12413936106985,
              "degrees": -179,
              "degMinSec": {},
              "vrc": "E179.00.000.000",
              "nats": "1790000W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9540163616679523,
                "degrees": 54.66111111,
                "percentage": 141.0319950539569,
                "degMinSec": {}
              },
              "radians": 0.9540163616679523,
              "degrees": 54.66111111,
              "degMinSec": {},
              "vrc": "N054.39.040.000",
              "nats": "543940N"
            },
            "lon": {
              "angle": {
                "radians": -0.10873401239924618,
                "degrees": -6.2299999999999685,
                "percentage": -10.916457257390238,
                "degMinSec": {}
              },
              "radians": -0.10873401239924618,
              "degrees": -6.2299999999999685,
              "degMinSec": {},
              "vrc": "E006.13.048.000",
              "nats": "0061348W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EZY28JM",
    "delayMs": 782556,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9532560803632874,
          "degrees": 54.6175502,
          "percentage": 140.80499015231928,
          "degMinSec": {}
        },
        "radians": 0.9532560803632874,
        "degrees": 54.6175502,
        "degMinSec": {},
        "vrc": "N054.37.003.181",
        "nats": "543703N"
      },
      "longitude": {
        "angle": {
          "radians": -0.10245083232805463,
          "degrees": -5.870000299999985,
          "percentage": -10.281079090017418,
          "degMinSec": {}
        },
        "radians": -0.10245083232805463,
        "degrees": -5.870000299999985,
        "degMinSec": {},
        "vrc": "E005.52.012.001",
        "nats": "0055212W"
      },
      "indicatedAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "trueAltitude": {
        "meters": 81.68639738603528,
        "feet": 268,
        "nauticalMiles": 0.04410712601837758,
        "statuteMiles": 0.050757574133333386
      },
      "pressureAltitude": {
        "meters": 19.605781136035283,
        "feet": 64.32343098235,
        "nauticalMiles": 0.010586274911466135,
        "statuteMiles": 0.012182467599242476
      },
      "densityAltitude": {
        "meters": -116.76888099363045,
        "feet": -383.1000155191425,
        "nauticalMiles": -0.06305015172442249,
        "statuteMiles": -0.07255681879923151
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.835262917116978,
          "degrees": 334.33593749999994,
          "percentage": -48.04952269840091,
          "degMinSec": {}
        },
        "radians": 5.835262917116978,
        "degrees": 334.33593749999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.80959891749545,
          "degrees": 332.865498636261,
          "percentage": -51.24859344565099,
          "degMinSec": {}
        },
        "radians": 5.80959891749545,
        "degrees": 332.865498636261
      },
      "track_True": {
        "angle": {
          "radians": 5.80959891749545,
          "degrees": 332.865498636261,
          "percentage": -51.24859344565099,
          "degMinSec": {}
        },
        "radians": 5.80959891749545,
        "degrees": 332.865498636261
      },
      "track_Mag": {
        "angle": {
          "radians": 5.835262917116978,
          "degrees": 334.33593749999994,
          "percentage": -48.04952269840091,
          "degMinSec": {}
        },
        "radians": 5.835262917116978,
        "degrees": 334.33593749999994
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "trueAirSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "groundSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "machNumber": 0,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -0,
        "knots": -0,
        "feetPerMinute": -0
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 102003.921875,
        "hectopascals": 1020.03921875,
        "inchesOfMercury": 30.125198427347904
      },
      "surfacePressure": {
        "pascals": 102003.921875,
        "hectopascals": 1020.03921875,
        "inchesOfMercury": 30.125198427347904
      },
      "windDirection": {
        "angle": {
          "radians": 2.888472549290217,
          "degrees": 165.4972863137231,
          "percentage": -25.86681155013803,
          "degMinSec": {}
        },
        "radians": 2.888472549290217,
        "degrees": 165.4972863137231
      },
      "windSpeed": {
        "metersPerSecond": 10.526210135989794,
        "knots": 20.461310415582943,
        "feetPerMinute": 2072.0886757536455
      },
      "windXComp": {
        "metersPerSecond": 2.3019205379196492,
        "knots": 4.474574426111882,
        "feetPerMinute": 453.1339786576981
      },
      "windHComp": {
        "metersPerSecond": -10.271429387583698,
        "knots": -19.966056386478243,
        "feetPerMinute": -2021.9349835176058
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9512044423369096,
            "degrees": 54.5,
            "percentage": 140.19482944763357,
            "degMinSec": {}
          },
          "radians": 0.9512044423369096,
          "degrees": 54.5,
          "degMinSec": {},
          "vrc": "N054.30.000.000",
          "nats": "543000N"
        },
        "lon": {
          "angle": {
            "radians": -0.10035696462189136,
            "degrees": -5.75003051757809,
            "percentage": -10.069524321626655,
            "degMinSec": {}
          },
          "radians": -0.10035696462189136,
          "degrees": -5.75003051757809,
          "degMinSec": {},
          "vrc": "E005.45.000.110",
          "nats": "0054500W"
        },
        "geoPotentialHeight": {
          "meters": 220.80575561523438,
          "feet": 724.4283552526855,
          "nauticalMiles": 0.11922556998662763,
          "statuteMiles": 0.13720233561950357
        },
        "levelPressure": {
          "pascals": 100000,
          "hectopascals": 1000,
          "inchesOfMercury": 29.533372711163615
        },
        "temp": {
          "kelvin": 285.0251159667969,
          "celsius": 11.875115966796898
        },
        "v": {
          "metersPerSecond": 10.190800666809082,
          "knots": 19.80932673137283,
          "feetPerMinute": 2006.0631875816346
        },
        "u": {
          "metersPerSecond": -2.636035203933716,
          "knots": -5.12404121495533,
          "feetPerMinute": -518.9045843084335
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102003.921875,
          "hectopascals": 1020.03921875,
          "inchesOfMercury": 30.125198427347904
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9532560803632874,
            "degrees": 54.6175502,
            "percentage": 140.80499015231928,
            "degMinSec": {}
          },
          "radians": 0.9532560803632874,
          "degrees": 54.6175502,
          "degMinSec": {},
          "vrc": "N054.37.003.181",
          "nats": "543703N"
        },
        "lon": {
          "angle": {
            "radians": -0.10245083232805463,
            "degrees": -5.870000299999985,
            "percentage": -10.281079090017418,
            "degMinSec": {}
          },
          "radians": -0.10245083232805463,
          "degrees": -5.870000299999985,
          "degMinSec": {},
          "vrc": "E005.52.012.001",
          "nats": "0055212W"
        },
        "alt": {
          "meters": 81.68639738603528,
          "feet": 268,
          "nauticalMiles": 0.04410712601837758,
          "statuteMiles": 0.050757574133333386
        }
      },
      "onGround": true,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9532559382614316,
                  "degrees": 54.6175420581634,
                  "percentage": 140.80494776896398,
                  "degMinSec": {}
                },
                "radians": 0.9532559382614316,
                "degrees": 54.6175420581634,
                "degMinSec": {},
                "vrc": "N054.37.003.151",
                "nats": "543703N"
              },
              "lon": {
                "angle": {
                  "radians": -0.1024513157449789,
                  "degrees": -5.8700279977494905,
                  "percentage": -10.2811279426868,
                  "degMinSec": {}
                },
                "radians": -0.1024513157449789,
                "degrees": -5.8700279977494905,
                "degMinSec": {},
                "vrc": "E005.52.012.101",
                "nats": "0055212W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPOD"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 2.6718099131429565,
            "degrees": 153.08343168430648,
            "percentage": -50.769260840582454,
            "degMinSec": {}
          },
          "radians": 2.6718099131429565,
          "degrees": 153.08343168430648
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 2.6760808853295313,
            "degrees": 153.32814036501497,
            "percentage": -50.23323757359711,
            "degMinSec": {}
          },
          "radians": 2.6760808853295313,
          "degrees": 153.32814036501497
        },
        "legLength": {
          "meters": 43143.982488125286,
          "feet": 141548.50350634096,
          "nauticalMiles": 23.295886872637844,
          "statuteMiles": 26.808427836513065
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9532559382614316,
                  "degrees": 54.6175420581634,
                  "percentage": 140.80494776896398,
                  "degMinSec": {}
                },
                "radians": 0.9532559382614316,
                "degrees": 54.6175420581634,
                "degMinSec": {},
                "vrc": "N054.37.003.151",
                "nats": "543703N"
              },
              "lon": {
                "angle": {
                  "radians": -0.1024513157449789,
                  "degrees": -5.8700279977494905,
                  "percentage": -10.2811279426868,
                  "degMinSec": {}
                },
                "radians": -0.1024513157449789,
                "degrees": -5.8700279977494905,
                "degMinSec": {},
                "vrc": "E005.52.012.101",
                "nats": "0055212W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "percentage": 139.01720103334867,
                  "degMinSec": {}
                },
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "degMinSec": {},
                "vrc": "N054.16.016.310",
                "nats": "541616N"
              },
              "lon": {
                "angle": {
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "percentage": -9.750884153790778,
                  "degMinSec": {}
                },
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "degMinSec": {},
                "vrc": "E005.34.009.260",
                "nats": "0053409W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 2.1949366717707015,
              "degrees": 125.76060759095286,
              "percentage": -138.85453951310936,
              "degMinSec": {}
            },
            "radians": 2.1949366717707015,
            "degrees": 125.76060759095286
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 2.2099233586737004,
              "degrees": 126.61928149937869,
              "percentage": -134.55549023720113,
              "degMinSec": {}
            },
            "radians": 2.2099233586737004,
            "degrees": 126.61928149937869
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "SOSIM"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPOD"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 85815.28881605434,
            "feet": 281546.2321592637,
            "nauticalMiles": 46.336549036746405,
            "statuteMiles": 53.323148323822835
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9472110804881934,
                    "degrees": 54.27119722,
                    "percentage": 139.01720103334867,
                    "degMinSec": {}
                  },
                  "radians": 0.9472110804881934,
                  "degrees": 54.27119722,
                  "degMinSec": {},
                  "vrc": "N054.16.016.310",
                  "nats": "541616N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09720155546061449,
                    "degrees": -5.569238890000011,
                    "percentage": -9.750884153790778,
                    "degMinSec": {}
                  },
                  "radians": -0.09720155546061449,
                  "degrees": -5.569238890000011,
                  "degMinSec": {},
                  "vrc": "E005.34.009.260",
                  "nats": "0053409W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 1.9445138144891772,
              "degrees": 111.41243477511456,
              "percentage": -255.00700170314428,
              "degMinSec": {}
            },
            "radians": 1.9445138144891772,
            "degrees": 111.41243477511456
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 1.9564011439449978,
              "degrees": 112.09352858261462,
              "percentage": -246.35012106966983,
              "degMinSec": {}
            },
            "radians": 1.9564011439449978,
            "degrees": 112.09352858261462
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9357751015108869,
                    "degrees": 53.61596389000001,
                    "percentage": 135.7158516082462,
                    "degMinSec": {}
                  },
                  "radians": 0.9357751015108869,
                  "degrees": 53.61596389000001,
                  "degMinSec": {},
                  "vrc": "N053.36.057.470",
                  "nats": "533657N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0639407188650516,
                    "degrees": -3.6635333299999795,
                    "percentage": -6.40280003476659,
                    "degMinSec": {}
                  },
                  "radians": -0.0639407188650516,
                  "degrees": -3.6635333299999795,
                  "degMinSec": {},
                  "vrc": "E003.39.048.720",
                  "nats": "0033949W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PENIL"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "SOSIM"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 59862.33239649776,
            "feet": 196398.7346197257,
            "nauticalMiles": 32.32307364821693,
            "statuteMiles": 37.19672885131939
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9392574211807754,
                    "degrees": 53.81548611,
                    "percentage": 136.71018811825857,
                    "degMinSec": {}
                  },
                  "radians": 0.9392574211807754,
                  "degrees": 53.81548611,
                  "degMinSec": {},
                  "vrc": "N053.48.055.750",
                  "nats": "534856N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07868749056761892,
                    "degrees": -4.508461110000039,
                    "percentage": -7.8850297468587085,
                    "degMinSec": {}
                  },
                  "radians": -0.07868749056761892,
                  "degrees": -4.508461110000039,
                  "degMinSec": {},
                  "vrc": "E004.30.030.460",
                  "nats": "0043030W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9357751015108869,
                    "degrees": 53.61596389000001,
                    "percentage": 135.7158516082462,
                    "degMinSec": {}
                  },
                  "radians": 0.9357751015108869,
                  "degrees": 53.61596389000001,
                  "degMinSec": {},
                  "vrc": "N053.36.057.470",
                  "nats": "533657N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.0639407188650516,
                    "degrees": -3.6635333299999795,
                    "percentage": -6.40280003476659,
                    "degMinSec": {}
                  },
                  "radians": -0.0639407188650516,
                  "degrees": -3.6635333299999795,
                  "degMinSec": {},
                  "vrc": "E003.39.048.720",
                  "nats": "0033949W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPOD; PEPOD =(TF)=> SOSIM; SOSIM =(TF)=> PENIL; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9532559382614316,
                "degrees": 54.6175420581634,
                "percentage": 140.80494776896398,
                "degMinSec": {}
              },
              "radians": 0.9532559382614316,
              "degrees": 54.6175420581634,
              "degMinSec": {},
              "vrc": "N054.37.003.151",
              "nats": "543703N"
            },
            "lon": {
              "angle": {
                "radians": -0.1024513157449789,
                "degrees": -5.8700279977494905,
                "percentage": -10.2811279426868,
                "degMinSec": {}
              },
              "radians": -0.1024513157449789,
              "degrees": -5.8700279977494905,
              "degMinSec": {},
              "vrc": "E005.52.012.101",
              "nats": "0055212W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9472110804881934,
                "degrees": 54.27119722,
                "percentage": 139.01720103334867,
                "degMinSec": {}
              },
              "radians": 0.9472110804881934,
              "degrees": 54.27119722,
              "degMinSec": {},
              "vrc": "N054.16.016.310",
              "nats": "541616N"
            },
            "lon": {
              "angle": {
                "radians": -0.09720155546061449,
                "degrees": -5.569238890000011,
                "percentage": -9.750884153790778,
                "degMinSec": {}
              },
              "radians": -0.09720155546061449,
              "degrees": -5.569238890000011,
              "degMinSec": {},
              "vrc": "E005.34.009.260",
              "nats": "0053409W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9392574211807754,
                "degrees": 53.81548611,
                "percentage": 136.71018811825857,
                "degMinSec": {}
              },
              "radians": 0.9392574211807754,
              "degrees": 53.81548611,
              "degMinSec": {},
              "vrc": "N053.48.055.750",
              "nats": "534856N"
            },
            "lon": {
              "angle": {
                "radians": -0.07868749056761892,
                "degrees": -4.508461110000039,
                "percentage": -7.8850297468587085,
                "degMinSec": {}
              },
              "radians": -0.07868749056761892,
              "degrees": -4.508461110000039,
              "degMinSec": {},
              "vrc": "E004.30.030.460",
              "nats": "0043030W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9392574211807754,
                "degrees": 53.81548611,
                "percentage": 136.71018811825857,
                "degMinSec": {}
              },
              "radians": 0.9392574211807754,
              "degrees": 53.81548611,
              "degMinSec": {},
              "vrc": "N053.48.055.750",
              "nats": "534856N"
            },
            "lon": {
              "angle": {
                "radians": -0.07868749056761892,
                "degrees": -4.508461110000039,
                "percentage": -7.8850297468587085,
                "degMinSec": {}
              },
              "radians": -0.07868749056761892,
              "degrees": -4.508461110000039,
              "degMinSec": {},
              "vrc": "E004.30.030.460",
              "nats": "0043030W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9357751015108869,
                "degrees": 53.61596389000001,
                "percentage": 135.7158516082462,
                "degMinSec": {}
              },
              "radians": 0.9357751015108869,
              "degrees": 53.61596389000001,
              "degMinSec": {},
              "vrc": "N053.36.057.470",
              "nats": "533657N"
            },
            "lon": {
              "angle": {
                "radians": -0.0639407188650516,
                "degrees": -3.6635333299999795,
                "percentage": -6.40280003476659,
                "degMinSec": {}
              },
              "radians": -0.0639407188650516,
              "degrees": -3.6635333299999795,
              "degMinSec": {},
              "vrc": "E003.39.048.720",
              "nats": "0033949W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 334,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.82939970166106,
          "degrees": 334,
          "percentage": -48.773258856586196,
          "degMinSec": {}
        },
        "radians": 5.82939970166106,
        "degrees": 334
      },
      "selectedAltitude": 3000,
      "selectedAltitudeLength": {
        "meters": 914.399970739201,
        "feet": 3000,
        "nauticalMiles": 0.4937364852803461,
        "statuteMiles": 0.5681818000000006
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "ON_GROUND"
  },
  {
    "callsign": "RUK94EF",
    "delayMs": 842570,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9423788935042152,
          "degrees": 53.9943333,
          "percentage": 137.60956931379638,
          "degMinSec": {}
        },
        "radians": 0.9423788935042152,
        "degrees": 53.9943333,
        "degMinSec": {},
        "vrc": "N053.59.039.600",
        "nats": "535940N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07346527153957183,
          "degrees": -4.209250000000029,
          "percentage": -7.359772509721122,
          "degMinSec": {}
        },
        "radians": -0.07346527153957183,
        "degrees": -4.209250000000029,
        "degMinSec": {},
        "vrc": "E004.12.033.300",
        "nats": "0041233W"
      },
      "indicatedAltitude": {
        "meters": 7176.163817163608,
        "feet": 23543.84529790305,
        "nauticalMiles": 3.874818475790285,
        "statuteMiles": 4.459061466761368
      },
      "trueAltitude": {
        "meters": 7315.199765913608,
        "feet": 24000,
        "nauticalMiles": 3.9498918822427687,
        "statuteMiles": 4.545454400000005
      },
      "pressureAltitude": {
        "meters": 7176.163817163608,
        "feet": 23543.84529790305,
        "nauticalMiles": 3.874818475790285,
        "statuteMiles": 4.459061466761368
      },
      "densityAltitude": {
        "meters": 7266.77955812549,
        "feet": 23841.141045480435,
        "nauticalMiles": 3.923747061622835,
        "statuteMiles": 4.515367477758323
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.386703218873489,
          "degrees": 308.6353599309862,
          "percentage": -125.10940119104139,
          "degMinSec": {}
        },
        "radians": 5.386703218873489,
        "degrees": 308.6353599309862
      },
      "track_True": {
        "angle": {
          "radians": 5.43118978187351,
          "degrees": 311.1842522359303,
          "percentage": -114.29244795736697,
          "degMinSec": {}
        },
        "radians": 5.43118978187351,
        "degrees": 311.1842522359303
      },
      "track_Mag": {
        "angle": {
          "radians": 5.444098936357477,
          "degrees": 311.923892304944,
          "percentage": -111.35836716347079,
          "degMinSec": {}
        },
        "radians": 5.444098936357477,
        "degrees": 311.923892304944
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 183.78161966303986,
        "knots": 357.24279869228207,
        "feetPerMinute": 36177.48534331726
      },
      "groundSpeed": {
        "metersPerSecond": 182.71730005915623,
        "knots": 355.17392741619045,
        "feetPerMinute": 35967.97360356493
      },
      "machNumber": 0.5807701735574268,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": -30.29639268073222,
        "knots": -58.89146113408524,
        "feetPerMinute": -5963.85701775921
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -180.18806933713066,
        "knots": -350.25749745256536,
        "feetPerMinute": -35470.0935242419
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.939898830195596,
          "degrees": 225.73957467873785,
          "percentage": 102.61550892967574,
          "degMinSec": {}
        },
        "radians": 3.939898830195596,
        "degrees": 225.73957467873785
      },
      "windSpeed": {
        "metersPerSecond": 8.605814710605769,
        "knots": 16.72836129032276,
        "feetPerMinute": 1694.0580681086299
      },
      "windXComp": {
        "metersPerSecond": 8.539746519310022,
        "knots": 16.599935033081668,
        "feetPerMinute": 1681.0525182247854
      },
      "windHComp": {
        "metersPerSecond": 1.0643196038836353,
        "knots": 2.0688712760915813,
        "feetPerMinute": 209.51173975233516
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 7477.01806640625,
          "feet": 24530.899952988282,
          "nauticalMiles": 4.037266774517414,
          "statuteMiles": 4.646003630302937
        },
        "levelPressure": {
          "pascals": 40000,
          "hectopascals": 400,
          "inchesOfMercury": 11.813349084465447
        },
        "temp": {
          "kelvin": 248.119384765625,
          "celsius": -25.030615234374977
        },
        "v": {
          "metersPerSecond": 6.006176948547363,
          "knots": 11.6750710243721,
          "feetPerMinute": 1182.318334792328
        },
        "u": {
          "metersPerSecond": 6.16326904296875,
          "knots": 11.980433549560546,
          "feetPerMinute": 1213.2419764160156
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9423788935042152,
            "degrees": 53.9943333,
            "percentage": 137.60956931379638,
            "degMinSec": {}
          },
          "radians": 0.9423788935042152,
          "degrees": 53.9943333,
          "degMinSec": {},
          "vrc": "N053.59.039.600",
          "nats": "535940N"
        },
        "lon": {
          "angle": {
            "radians": -0.07346527153957183,
            "degrees": -4.209250000000029,
            "percentage": -7.359772509721122,
            "degMinSec": {}
          },
          "radians": -0.07346527153957183,
          "degrees": -4.209250000000029,
          "degMinSec": {},
          "vrc": "E004.12.033.300",
          "nats": "0041233W"
        },
        "alt": {
          "meters": 7315.199765913608,
          "feet": 24000,
          "nauticalMiles": 3.9498918822427687,
          "statuteMiles": 4.545454400000005
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": null,
      "routeLegs": [],
      "asString": "",
      "fmsLines": [],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EAG19E",
    "delayMs": 903214,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "percentage": 137.3334254963888,
          "degMinSec": {}
        },
        "radians": 0.9414233257387483,
        "degrees": 53.9395833,
        "degMinSec": {},
        "vrc": "N053.56.022.500",
        "nats": "535622N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "percentage": -7.561162063112771,
          "degMinSec": {}
        },
        "radians": -0.07546801940294223,
        "degrees": -4.323998999999997,
        "degMinSec": {},
        "vrc": "E004.19.026.396",
        "nats": "0041926W"
      },
      "indicatedAltitude": {
        "meters": 4737.7638951924055,
        "feet": 15543.845297903052,
        "nauticalMiles": 2.5581878483760288,
        "statuteMiles": 2.9439100000947005
      },
      "trueAltitude": {
        "meters": 4876.7998439424055,
        "feet": 16000.000000000002,
        "nauticalMiles": 2.6332612548285126,
        "statuteMiles": 3.0303029333333367
      },
      "pressureAltitude": {
        "meters": 4737.7638951924055,
        "feet": 15543.845297903052,
        "nauticalMiles": 2.5581878483760288,
        "statuteMiles": 2.9439100000947005
      },
      "densityAltitude": {
        "meters": 4944.009455553312,
        "feet": 16220.50398215753,
        "nauticalMiles": 2.6695515418754385,
        "statuteMiles": 3.072065049829814
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.386033308863389,
          "degrees": 308.59697691475395,
          "percentage": -125.28139310935545,
          "degMinSec": {}
        },
        "radians": 5.386033308863389,
        "degrees": 308.59697691475395
      },
      "track_True": {
        "angle": {
          "radians": 5.414304845068882,
          "degrees": 310.21681661968,
          "percentage": -118.26359443586976,
          "degMinSec": {}
        },
        "radians": 5.414304845068882,
        "degrees": 310.21681661968
      },
      "track_Mag": {
        "angle": {
          "radians": 5.427883909562951,
          "degrees": 310.9948397049261,
          "percentage": -115.05776791942601,
          "degMinSec": {}
        },
        "radians": 5.427883909562951,
        "degrees": 310.9948397049261
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 163.24091200988337,
        "knots": 317.31486736493974,
        "feetPerMinute": 32134.038825510346
      },
      "groundSpeed": {
        "metersPerSecond": 166.612786856219,
        "knots": 323.86926605374015,
        "feetPerMinute": 32797.79373776145
      },
      "machNumber": 0.49940684732212365,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 119.62002830374654,
        "knots": 232.52267429806787,
        "feetPerMinute": 23547.250419603828
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -115.97788398055356,
        "knots": -225.44291390829514,
        "feetPerMinute": -22830.29285272556
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.190543799625692,
          "degrees": 182.80469407018555,
          "percentage": 4.8990282700142105,
          "degMinSec": {}
        },
        "radians": 3.190543799625692,
        "degrees": 182.80469407018555
      },
      "windSpeed": {
        "metersPerSecond": 5.765382425157939,
        "knots": 11.207004034848708,
        "feetPerMinute": 1134.9178365453104
      },
      "windXComp": {
        "metersPerSecond": 4.676547287152038,
        "knots": 9.090478384846767,
        "feetPerMinute": 920.5802040947937
      },
      "windHComp": {
        "metersPerSecond": -3.371874846335633,
        "knots": -6.554398688800442,
        "feetPerMinute": -663.7549122511078
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 5088.79541015625,
          "feet": 16695.52353345703,
          "nauticalMiles": 2.7477297031081265,
          "statuteMiles": 3.162030871060662
        },
        "levelPressure": {
          "pascals": 55000,
          "hectopascals": 550,
          "inchesOfMercury": 16.243354991139988
        },
        "temp": {
          "kelvin": 264.48101806640625,
          "celsius": -8.668981933593727
        },
        "v": {
          "metersPerSecond": 5.758476257324219,
          "knots": 11.193579521942139,
          "feetPerMinute": 1133.5583546447754
        },
        "u": {
          "metersPerSecond": 0.2821093797683716,
          "knots": 0.5483766252064705,
          "feetPerMinute": 55.53334425115585
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9414233257387483,
            "degrees": 53.9395833,
            "percentage": 137.3334254963888,
            "degMinSec": {}
          },
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "degMinSec": {},
          "vrc": "N053.56.022.500",
          "nats": "535622N"
        },
        "lon": {
          "angle": {
            "radians": -0.07546801940294223,
            "degrees": -4.323998999999997,
            "percentage": -7.561162063112771,
            "degMinSec": {}
          },
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "degMinSec": {},
          "vrc": "E004.19.026.396",
          "nats": "0041926W"
        },
        "alt": {
          "meters": 4876.7998439424055,
          "feet": 16000.000000000002,
          "nauticalMiles": 2.6332612548285126,
          "statuteMiles": 3.0303029333333367
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9414257432551684,
                  "degrees": 53.93972181348777,
                  "percentage": 137.33412320530056,
                  "degMinSec": {}
                },
                "radians": 0.9414257432551684,
                "degrees": 53.93972181348777,
                "degMinSec": {},
                "vrc": "N053.56.022.999",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546517703939593,
                  "degrees": -4.323836144564951,
                  "percentage": -7.56087620180703,
                  "degMinSec": {}
                },
                "radians": -0.07546517703939593,
                "degrees": -4.323836144564951,
                "degMinSec": {},
                "vrc": "E004.19.025.810",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPEG"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.317175514746638,
            "degrees": 304.6517159252834,
            "percentage": -144.67869151467332,
            "degMinSec": {}
          },
          "radians": 5.317175514746638,
          "degrees": 304.6517159252834
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.314432683873747,
            "degrees": 304.4945632923485,
            "percentage": -145.53048416750815,
            "degMinSec": {}
          },
          "radians": 5.314432683873747,
          "degrees": 304.4945632923485
        },
        "legLength": {
          "meters": 15430.795914969682,
          "feet": 50625.972469669134,
          "nauticalMiles": 8.331963237024667,
          "statuteMiles": 9.588252054855694
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9414257432551684,
                  "degrees": 53.93972181348777,
                  "percentage": 137.33412320530056,
                  "degMinSec": {}
                },
                "radians": 0.9414257432551684,
                "degrees": 53.93972181348777,
                "degMinSec": {},
                "vrc": "N053.56.022.999",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546517703939593,
                  "degrees": -4.323836144564951,
                  "percentage": -7.56087620180703,
                  "degMinSec": {}
                },
                "radians": -0.07546517703939593,
                "degrees": -4.323836144564951,
                "degMinSec": {},
                "vrc": "E004.19.025.810",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.455962031115604,
              "degrees": 312.60359756654844,
              "percentage": -108.73547930832437,
              "degMinSec": {}
            },
            "radians": 5.455962031115604,
            "degrees": 312.60359756654844
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.444396015875018,
              "degrees": 311.9409137074789,
              "percentage": -111.29184132007772,
              "degMinSec": {}
            },
            "radians": 5.444396015875018,
            "degrees": 311.9409137074789
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPEG"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 71726.7717842852,
            "feet": 235324.06194075424,
            "nauticalMiles": 38.72935841484082,
            "statuteMiles": 44.56894969893646
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.848296457408213,
              "degrees": 335.0827043508014,
              "percentage": -46.45515050390753,
              "degMinSec": {}
            },
            "radians": 5.848296457408213,
            "degrees": 335.0827043508014
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.844404109141495,
              "degrees": 334.85968922272343,
              "percentage": -46.92924500944571,
              "degMinSec": {}
            },
            "radians": 5.844404109141495,
            "degrees": 334.85968922272343
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAGEE"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 41621.00125975992,
            "feet": 136551.84577307073,
            "nauticalMiles": 22.47354279684661,
            "statuteMiles": 25.8620911748886
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPEG; PEPEG =(TF)=> ROBOP; ROBOP =(TF)=> MAGEE; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9414257432551684,
                "degrees": 53.93972181348777,
                "percentage": 137.33412320530056,
                "degMinSec": {}
              },
              "radians": 0.9414257432551684,
              "degrees": 53.93972181348777,
              "degMinSec": {},
              "vrc": "N053.56.022.999",
              "nats": "535623N"
            },
            "lon": {
              "angle": {
                "radians": -0.07546517703939593,
                "degrees": -4.323836144564951,
                "percentage": -7.56087620180703,
                "degMinSec": {}
              },
              "radians": -0.07546517703939593,
              "degrees": -4.323836144564951,
              "degMinSec": {},
              "vrc": "E004.19.025.810",
              "nats": "0041926W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EAG49BG",
    "delayMs": 1023860,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "percentage": 137.3334254963888,
          "degMinSec": {}
        },
        "radians": 0.9414233257387483,
        "degrees": 53.9395833,
        "degMinSec": {},
        "vrc": "N053.56.022.500",
        "nats": "535622N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "percentage": -7.561162063112771,
          "degMinSec": {}
        },
        "radians": -0.07546801940294223,
        "degrees": -4.323998999999997,
        "degMinSec": {},
        "vrc": "E004.19.026.396",
        "nats": "0041926W"
      },
      "indicatedAltitude": {
        "meters": 4737.7638951924055,
        "feet": 15543.845297903052,
        "nauticalMiles": 2.5581878483760288,
        "statuteMiles": 2.9439100000947005
      },
      "trueAltitude": {
        "meters": 4876.7998439424055,
        "feet": 16000.000000000002,
        "nauticalMiles": 2.6332612548285126,
        "statuteMiles": 3.0303029333333367
      },
      "pressureAltitude": {
        "meters": 4737.7638951924055,
        "feet": 15543.845297903052,
        "nauticalMiles": 2.5581878483760288,
        "statuteMiles": 2.9439100000947005
      },
      "densityAltitude": {
        "meters": 4944.009455553312,
        "feet": 16220.50398215753,
        "nauticalMiles": 2.6695515418754385,
        "statuteMiles": 3.072065049829814
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.386033308863389,
          "degrees": 308.59697691475395,
          "percentage": -125.28139310935545,
          "degMinSec": {}
        },
        "radians": 5.386033308863389,
        "degrees": 308.59697691475395
      },
      "track_True": {
        "angle": {
          "radians": 5.414304845068882,
          "degrees": 310.21681661968,
          "percentage": -118.26359443586976,
          "degMinSec": {}
        },
        "radians": 5.414304845068882,
        "degrees": 310.21681661968
      },
      "track_Mag": {
        "angle": {
          "radians": 5.427883909562951,
          "degrees": 310.9948397049261,
          "percentage": -115.05776791942601,
          "degMinSec": {}
        },
        "radians": 5.427883909562951,
        "degrees": 310.9948397049261
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 163.24091200988337,
        "knots": 317.31486736493974,
        "feetPerMinute": 32134.038825510346
      },
      "groundSpeed": {
        "metersPerSecond": 166.612786856219,
        "knots": 323.86926605374015,
        "feetPerMinute": 32797.79373776145
      },
      "machNumber": 0.49940684732212365,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 119.62002830374654,
        "knots": 232.52267429806787,
        "feetPerMinute": 23547.250419603828
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -115.97788398055356,
        "knots": -225.44291390829514,
        "feetPerMinute": -22830.29285272556
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.190543799625692,
          "degrees": 182.80469407018555,
          "percentage": 4.8990282700142105,
          "degMinSec": {}
        },
        "radians": 3.190543799625692,
        "degrees": 182.80469407018555
      },
      "windSpeed": {
        "metersPerSecond": 5.765382425157939,
        "knots": 11.207004034848708,
        "feetPerMinute": 1134.9178365453104
      },
      "windXComp": {
        "metersPerSecond": 4.676547287152038,
        "knots": 9.090478384846767,
        "feetPerMinute": 920.5802040947937
      },
      "windHComp": {
        "metersPerSecond": -3.371874846335633,
        "knots": -6.554398688800442,
        "feetPerMinute": -663.7549122511078
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 5088.79541015625,
          "feet": 16695.52353345703,
          "nauticalMiles": 2.7477297031081265,
          "statuteMiles": 3.162030871060662
        },
        "levelPressure": {
          "pascals": 55000,
          "hectopascals": 550,
          "inchesOfMercury": 16.243354991139988
        },
        "temp": {
          "kelvin": 264.48101806640625,
          "celsius": -8.668981933593727
        },
        "v": {
          "metersPerSecond": 5.758476257324219,
          "knots": 11.193579521942139,
          "feetPerMinute": 1133.5583546447754
        },
        "u": {
          "metersPerSecond": 0.2821093797683716,
          "knots": 0.5483766252064705,
          "feetPerMinute": 55.53334425115585
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9414233257387483,
            "degrees": 53.9395833,
            "percentage": 137.3334254963888,
            "degMinSec": {}
          },
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "degMinSec": {},
          "vrc": "N053.56.022.500",
          "nats": "535622N"
        },
        "lon": {
          "angle": {
            "radians": -0.07546801940294223,
            "degrees": -4.323998999999997,
            "percentage": -7.561162063112771,
            "degMinSec": {}
          },
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "degMinSec": {},
          "vrc": "E004.19.026.396",
          "nats": "0041926W"
        },
        "alt": {
          "meters": 4876.7998439424055,
          "feet": 16000.000000000002,
          "nauticalMiles": 2.6332612548285126,
          "statuteMiles": 3.0303029333333367
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9414257432551684,
                  "degrees": 53.93972181348777,
                  "percentage": 137.33412320530056,
                  "degMinSec": {}
                },
                "radians": 0.9414257432551684,
                "degrees": 53.93972181348777,
                "degMinSec": {},
                "vrc": "N053.56.022.999",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546517703939593,
                  "degrees": -4.323836144564951,
                  "percentage": -7.56087620180703,
                  "degMinSec": {}
                },
                "radians": -0.07546517703939593,
                "degrees": -4.323836144564951,
                "degMinSec": {},
                "vrc": "E004.19.025.810",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPEG"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.317175514746638,
            "degrees": 304.6517159252834,
            "percentage": -144.67869151467332,
            "degMinSec": {}
          },
          "radians": 5.317175514746638,
          "degrees": 304.6517159252834
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.314432683873747,
            "degrees": 304.4945632923485,
            "percentage": -145.53048416750815,
            "degMinSec": {}
          },
          "radians": 5.314432683873747,
          "degrees": 304.4945632923485
        },
        "legLength": {
          "meters": 15430.795914969682,
          "feet": 50625.972469669134,
          "nauticalMiles": 8.331963237024667,
          "statuteMiles": 9.588252054855694
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9414257432551684,
                  "degrees": 53.93972181348777,
                  "percentage": 137.33412320530056,
                  "degMinSec": {}
                },
                "radians": 0.9414257432551684,
                "degrees": 53.93972181348777,
                "degMinSec": {},
                "vrc": "N053.56.022.999",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546517703939593,
                  "degrees": -4.323836144564951,
                  "percentage": -7.56087620180703,
                  "degMinSec": {}
                },
                "radians": -0.07546517703939593,
                "degrees": -4.323836144564951,
                "degMinSec": {},
                "vrc": "E004.19.025.810",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.455962031115604,
              "degrees": 312.60359756654844,
              "percentage": -108.73547930832437,
              "degMinSec": {}
            },
            "radians": 5.455962031115604,
            "degrees": 312.60359756654844
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.444396015875018,
              "degrees": 311.9409137074789,
              "percentage": -111.29184132007772,
              "degMinSec": {}
            },
            "radians": 5.444396015875018,
            "degrees": 311.9409137074789
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPEG"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 71726.7717842852,
            "feet": 235324.06194075424,
            "nauticalMiles": 38.72935841484082,
            "statuteMiles": 44.56894969893646
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.848296457408213,
              "degrees": 335.0827043508014,
              "percentage": -46.45515050390753,
              "degMinSec": {}
            },
            "radians": 5.848296457408213,
            "degrees": 335.0827043508014
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.844404109141495,
              "degrees": 334.85968922272343,
              "percentage": -46.92924500944571,
              "degMinSec": {}
            },
            "radians": 5.844404109141495,
            "degrees": 334.85968922272343
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAGEE"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 41621.00125975992,
            "feet": 136551.84577307073,
            "nauticalMiles": 22.47354279684661,
            "statuteMiles": 25.8620911748886
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPEG; PEPEG =(TF)=> ROBOP; ROBOP =(TF)=> MAGEE; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9414257432551684,
                "degrees": 53.93972181348777,
                "percentage": 137.33412320530056,
                "degMinSec": {}
              },
              "radians": 0.9414257432551684,
              "degrees": 53.93972181348777,
              "degMinSec": {},
              "vrc": "N053.56.022.999",
              "nats": "535623N"
            },
            "lon": {
              "angle": {
                "radians": -0.07546517703939593,
                "degrees": -4.323836144564951,
                "percentage": -7.56087620180703,
                "degMinSec": {}
              },
              "radians": -0.07546517703939593,
              "degrees": -4.323836144564951,
              "degMinSec": {},
              "vrc": "E004.19.025.810",
              "nats": "0041926W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  },
  {
    "callsign": "EAG79A",
    "delayMs": 1084510,
    "simState": {
      "simRate": 8,
      "paused": false
    },
    "flightPlan": {
      "flightRules": "DVFR",
      "aircraftType": null,
      "filedTas": 0,
      "origin": null,
      "estimatedDepTime": 0,
      "actualDepTime": 0,
      "cruiseLevel": 0,
      "destination": null,
      "hoursEnroute": 0,
      "minutesEnroute": 0,
      "hoursFuel": 0,
      "minutesFuel": 0,
      "alternate": null,
      "remarks": null,
      "route": null
    },
    "xpdrMode": "ModeC",
    "squawk": 0,
    "position": {
      "latitude": {
        "angle": {
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "percentage": 137.3334254963888,
          "degMinSec": {}
        },
        "radians": 0.9414233257387483,
        "degrees": 53.9395833,
        "degMinSec": {},
        "vrc": "N053.56.022.500",
        "nats": "535622N"
      },
      "longitude": {
        "angle": {
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "percentage": -7.561162063112771,
          "degMinSec": {}
        },
        "radians": -0.07546801940294223,
        "degrees": -4.323998999999997,
        "degMinSec": {},
        "vrc": "E004.19.026.396",
        "nats": "0041926W"
      },
      "indicatedAltitude": {
        "meters": 4128.163914699604,
        "feet": 13543.845297903048,
        "nauticalMiles": 2.2290301915224644,
        "statuteMiles": 2.5651221334280327
      },
      "trueAltitude": {
        "meters": 4267.199863449604,
        "feet": 13999.999999999998,
        "nauticalMiles": 2.3041035979749482,
        "statuteMiles": 2.651515066666669
      },
      "pressureAltitude": {
        "meters": 4128.163914699604,
        "feet": 13543.845297903048,
        "nauticalMiles": 2.2290301915224644,
        "statuteMiles": 2.5651221334280327
      },
      "densityAltitude": {
        "meters": 4372.169617160849,
        "feet": 14344.388966766,
        "nauticalMiles": 2.3607827306484066,
        "statuteMiles": 2.716740247679085
      },
      "heading_Mag": {
        "angle": {
          "radians": 5.399612373357456,
          "degrees": 309.37499999999994,
          "percentage": -121.85035255879781,
          "degMinSec": {}
        },
        "radians": 5.399612373357456,
        "degrees": 309.37499999999994
      },
      "heading_True": {
        "angle": {
          "radians": 5.386033308863389,
          "degrees": 308.59697691475395,
          "percentage": -125.28139310935545,
          "degMinSec": {}
        },
        "radians": 5.386033308863389,
        "degrees": 308.59697691475395
      },
      "track_True": {
        "angle": {
          "radians": 5.419548066232178,
          "degrees": 310.5172310633905,
          "percentage": -117.01367988702715,
          "degMinSec": {}
        },
        "radians": 5.419548066232178,
        "degrees": 310.5172310633905
      },
      "track_Mag": {
        "angle": {
          "radians": 5.433127130726245,
          "degrees": 311.2952541486365,
          "percentage": -113.84662836552306,
          "degMinSec": {}
        },
        "radians": 5.433127130726245,
        "degrees": 311.2952541486365
      },
      "bank": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "pitch": {
        "radians": 0.04363323129985824,
        "degrees": 2.5,
        "percentage": 4.366094290851206,
        "degMinSec": {}
      },
      "indicatedAirSpeed": {
        "metersPerSecond": 128.61114369260085,
        "knots": 250,
        "feetPerMinute": 25317.155080345954
      },
      "trueAirSpeed": {
        "metersPerSecond": 158.65499611943326,
        "knots": 308.4005622767836,
        "feetPerMinute": 31231.299448108886
      },
      "groundSpeed": {
        "metersPerSecond": 163.7519421466034,
        "knots": 318.3082302300221,
        "feetPerMinute": 32234.635312335733
      },
      "machNumber": 0.481431403732183,
      "verticalSpeed": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "flightPathAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "velocity_X": {
        "metersPerSecond": 78.57032194114733,
        "knots": 152.72844888336758,
        "feetPerMinute": 15466.599302243629
      },
      "velocity_Y": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "velocity_Z": {
        "metersPerSecond": -143.67116296198415,
        "knots": -279.2743280966751,
        "feetPerMinute": -28281.725897531767
      },
      "heading_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "bank_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitch_Velocity": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "forward_Acceleration": {
        "metersPerSecondSquared": 0,
        "knotsPerSecond": 0
      },
      "altimeterSetting": {
        "pascals": 101325,
        "hectopascals": 1013.25,
        "inchesOfMercury": 29.924689899586532
      },
      "surfacePressure": {
        "pascals": 102845.515625,
        "hectopascals": 1028.45515625,
        "inchesOfMercury": 30.373749446249263
      },
      "windDirection": {
        "angle": {
          "radians": 3.0546033352409943,
          "degrees": 175.0158791958939,
          "percentage": -8.720940470315483,
          "degMinSec": {}
        },
        "radians": 3.0546033352409943,
        "degrees": 175.0158791958939
      },
      "windSpeed": {
        "metersPerSecond": 7.39351535381342,
        "knots": 14.371840459418094,
        "feetPerMinute": 1455.4164548043132
      },
      "windXComp": {
        "metersPerSecond": 5.355857586156434,
        "knots": 10.410951633704666,
        "feetPerMinute": 1054.3027081779285
      },
      "windHComp": {
        "metersPerSecond": -5.096946027170129,
        "knots": -9.907667953238493,
        "feetPerMinute": -1003.3358642268507
      },
      "gribPoint": {
        "lat": {
          "angle": {
            "radians": 0.9424777960769379,
            "degrees": 54,
            "percentage": 137.63819204711734,
            "degMinSec": {}
          },
          "radians": 0.9424777960769379,
          "degrees": 54,
          "degMinSec": {},
          "vrc": "N054.00.000.000",
          "nats": "540000N"
        },
        "lon": {
          "angle": {
            "radians": -0.07417596057754139,
            "degrees": -4.249969482421898,
            "percentage": -7.431230116756623,
            "degMinSec": {}
          },
          "radians": -0.07417596057754139,
          "degrees": -4.249969482421898,
          "degMinSec": {},
          "vrc": "E004.14.059.890",
          "nats": "0041500W"
        },
        "geoPotentialHeight": {
          "meters": 4408.77783203125,
          "feet": 14464.494662441406,
          "nauticalMiles": 2.3805495853300487,
          "statuteMiles": 2.7394875377987864
        },
        "levelPressure": {
          "pascals": 60000,
          "hectopascals": 600,
          "inchesOfMercury": 17.72002362669817
        },
        "temp": {
          "kelvin": 269.3143615722656,
          "celsius": -3.8356384277343523
        },
        "v": {
          "metersPerSecond": 7.365559101104736,
          "knots": 14.317497865327834,
          "feetPerMinute": 1449.9132552761077
        },
        "u": {
          "metersPerSecond": -0.6423460245132446,
          "knots": -1.2486204656739235,
          "feetPerMinute": -126.44607186384201
        },
        "wind": {},
        "relativeHumidity": 0,
        "sfcPress": {
          "pascals": 102845.515625,
          "hectopascals": 1028.45515625,
          "inchesOfMercury": 30.373749446249263
        }
      },
      "positionGeoPoint": {
        "lat": {
          "angle": {
            "radians": 0.9414233257387483,
            "degrees": 53.9395833,
            "percentage": 137.3334254963888,
            "degMinSec": {}
          },
          "radians": 0.9414233257387483,
          "degrees": 53.9395833,
          "degMinSec": {},
          "vrc": "N053.56.022.500",
          "nats": "535622N"
        },
        "lon": {
          "angle": {
            "radians": -0.07546801940294223,
            "degrees": -4.323998999999997,
            "percentage": -7.561162063112771,
            "degMinSec": {}
          },
          "radians": -0.07546801940294223,
          "degrees": -4.323998999999997,
          "degMinSec": {},
          "vrc": "E004.19.026.396",
          "nats": "0041926W"
        },
        "alt": {
          "meters": 4267.199863449604,
          "feet": 13999.999999999998,
          "nauticalMiles": 2.3041035979749482,
          "statuteMiles": 2.651515066666669
        }
      },
      "onGround": false,
      "bankRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "pitchRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      },
      "yawRate": {
        "radiansPerSecond": 0,
        "degreesPerSecond": 0
      }
    },
    "data": {
      "config": 0,
      "thrustLeverPos": 0,
      "thrustLeverVel": 0,
      "thrustReverse": false,
      "speedBrakePos": 0,
      "mass_kg": 60300,
      "aircraftConfig": {}
    },
    "fms": {
      "suspended": false,
      "cruiseAltitude": 0,
      "departureAirport": null,
      "arrivalAirport": null,
      "activeLeg": {
        "startPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9414255069719641,
                  "degrees": 53.939708275457406,
                  "percentage": 137.33405501242717,
                  "degMinSec": {}
                },
                "radians": 0.9414255069719641,
                "degrees": 53.939708275457406,
                "degMinSec": {},
                "vrc": "N053.56.022.950",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546545452234454,
                  "degrees": -4.323852043166794,
                  "percentage": -7.560904108730735,
                  "degMinSec": {}
                },
                "radians": -0.07546545452234454,
                "degrees": -4.323852043166794,
                "degMinSec": {},
                "vrc": "E004.19.025.867",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "*DIRECT"
          },
          "pointType": "FLY_OVER",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": -1,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "endPoint": {
          "point": {
            "pointPosition": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "pointName": "PEPEG"
          },
          "pointType": "FLY_BY",
          "lowerAltitudeConstraint": 0,
          "upperAltitudeConstraint": 0,
          "angleConstraint": -1,
          "vnavTargetAltitude": 0,
          "speedConstraintType": "FREE",
          "speedConstraint": 0
        },
        "initialTrueCourse": {
          "angle": {
            "radians": 5.317293885856314,
            "degrees": 304.65849809028407,
            "percentage": -144.64208332251917,
            "degMinSec": {}
          },
          "radians": 5.317293885856314,
          "degrees": 304.65849809028407
        },
        "finalTrueCourse": {
          "angle": {
            "radians": 5.314551279648036,
            "degrees": 304.5013583296834,
            "percentage": -145.4935134272846,
            "degMinSec": {}
          },
          "radians": 5.314551279648036,
          "degrees": 304.5013583296834
        },
        "legLength": {
          "meters": 15430.795914971715,
          "feet": 50625.972469675806,
          "nauticalMiles": 8.331963237025764,
          "statuteMiles": 9.588252054856957
        },
        "legType": "DIRECT_TO_FIX",
        "uiLines": [
          {
            "startPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9414255069719641,
                  "degrees": 53.939708275457406,
                  "percentage": 137.33405501242717,
                  "degMinSec": {}
                },
                "radians": 0.9414255069719641,
                "degrees": 53.939708275457406,
                "degMinSec": {},
                "vrc": "N053.56.022.950",
                "nats": "535623N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07546545452234454,
                  "degrees": -4.323852043166794,
                  "percentage": -7.560904108730735,
                  "degMinSec": {}
                },
                "radians": -0.07546545452234454,
                "degrees": -4.323852043166794,
                "degMinSec": {},
                "vrc": "E004.19.025.867",
                "nats": "0041926W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            },
            "endPoint": {
              "lat": {
                "angle": {
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "percentage": 137.73153606063852,
                  "degMinSec": {}
                },
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "degMinSec": {},
                "vrc": "N054.01.006.490",
                "nats": "540106N"
              },
              "lon": {
                "angle": {
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "percentage": -7.902025902955326,
                  "degMinSec": {}
                },
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "degMinSec": {},
                "vrc": "E004.31.005.300",
                "nats": "0043105W"
              },
              "alt": {
                "meters": 0,
                "feet": 0,
                "nauticalMiles": 0,
                "statuteMiles": 0
              }
            }
          }
        ]
      },
      "routeLegs": [
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.455962031115604,
              "degrees": 312.60359756654844,
              "percentage": -108.73547930832437,
              "degMinSec": {}
            },
            "radians": 5.455962031115604,
            "degrees": 312.60359756654844
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.444396015875018,
              "degrees": 311.9409137074789,
              "percentage": -111.29184132007772,
              "degMinSec": {}
            },
            "radians": 5.444396015875018,
            "degrees": 311.9409137074789
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "PEPEG"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 71726.7717842852,
            "feet": 235324.06194075424,
            "nauticalMiles": 38.72935841484082,
            "statuteMiles": 44.56894969893646
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9428001486159374,
                    "degrees": 54.01846944,
                    "percentage": 137.73153606063852,
                    "degMinSec": {}
                  },
                  "radians": 0.9428001486159374,
                  "degrees": 54.01846944,
                  "degMinSec": {},
                  "vrc": "N054.01.006.490",
                  "nats": "540106N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.07885639969290104,
                    "degrees": -4.518138889999951,
                    "percentage": -7.902025902955326,
                    "degMinSec": {}
                  },
                  "radians": -0.07885639969290104,
                  "degrees": -4.518138889999951,
                  "degMinSec": {},
                  "vrc": "E004.31.005.300",
                  "nats": "0043105W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        },
        {
          "legType": "TRACK_TO_FIX",
          "initialTrueCourse": {
            "angle": {
              "radians": 5.848296457408213,
              "degrees": 335.0827043508014,
              "percentage": -46.45515050390753,
              "degMinSec": {}
            },
            "radians": 5.848296457408213,
            "degrees": 335.0827043508014
          },
          "finalTrueCourse": {
            "angle": {
              "radians": 5.844404109141495,
              "degrees": 334.85968922272343,
              "percentage": -46.92924500944571,
              "degMinSec": {}
            },
            "radians": 5.844404109141495,
            "degrees": 334.85968922272343
          },
          "endPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "MAGEE"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "startPoint": {
            "point": {
              "pointPosition": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "pointName": "ROBOP"
            },
            "pointType": "FLY_BY",
            "lowerAltitudeConstraint": 0,
            "upperAltitudeConstraint": 0,
            "angleConstraint": -1,
            "vnavTargetAltitude": 0,
            "speedConstraintType": "FREE",
            "speedConstraint": 0
          },
          "legLength": {
            "meters": 41621.00125975992,
            "feet": 136551.84577307073,
            "nauticalMiles": 22.47354279684661,
            "statuteMiles": 25.8620911748886
          },
          "uiLines": [
            {
              "startPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.9503732778008005,
                    "degrees": 54.45237778,
                    "percentage": 139.94863782854935,
                    "degMinSec": {}
                  },
                  "radians": 0.9503732778008005,
                  "degrees": 54.45237778,
                  "degMinSec": {},
                  "vrc": "N054.27.008.560",
                  "nats": "542709N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09311016422740082,
                    "degrees": -5.334819440000042,
                    "percentage": -9.338017354418039,
                    "degMinSec": {}
                  },
                  "radians": -0.09311016422740082,
                  "degrees": -5.334819440000042,
                  "degMinSec": {},
                  "vrc": "E005.20.005.350",
                  "nats": "0052005W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              },
              "endPoint": {
                "lat": {
                  "angle": {
                    "radians": 0.956292707344866,
                    "degrees": 54.79153611,
                    "percentage": 141.71458787308552,
                    "degMinSec": {}
                  },
                  "radians": 0.956292707344866,
                  "degrees": 54.79153611,
                  "degMinSec": {},
                  "vrc": "N054.47.029.530",
                  "nats": "544730N"
                },
                "lon": {
                  "angle": {
                    "radians": -0.09788397919814429,
                    "degrees": -5.608338890000012,
                    "percentage": -9.819779969455576,
                    "degMinSec": {}
                  },
                  "radians": -0.09788397919814429,
                  "degrees": -5.608338890000012,
                  "degMinSec": {},
                  "vrc": "E005.36.030.020",
                  "nats": "0053630W"
                },
                "alt": {
                  "meters": 0,
                  "feet": 0,
                  "nauticalMiles": 0,
                  "statuteMiles": 0
                }
              }
            }
          ]
        }
      ],
      "asString": "PPOS =(DF)=> PEPEG; PEPEG =(TF)=> ROBOP; ROBOP =(TF)=> MAGEE; ",
      "fmsLines": [
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9414255069719641,
                "degrees": 53.939708275457406,
                "percentage": 137.33405501242717,
                "degMinSec": {}
              },
              "radians": 0.9414255069719641,
              "degrees": 53.939708275457406,
              "degMinSec": {},
              "vrc": "N053.56.022.950",
              "nats": "535623N"
            },
            "lon": {
              "angle": {
                "radians": -0.07546545452234454,
                "degrees": -4.323852043166794,
                "percentage": -7.560904108730735,
                "degMinSec": {}
              },
              "radians": -0.07546545452234454,
              "degrees": -4.323852043166794,
              "degMinSec": {},
              "vrc": "E004.19.025.867",
              "nats": "0041926W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9428001486159374,
                "degrees": 54.01846944,
                "percentage": 137.73153606063852,
                "degMinSec": {}
              },
              "radians": 0.9428001486159374,
              "degrees": 54.01846944,
              "degMinSec": {},
              "vrc": "N054.01.006.490",
              "nats": "540106N"
            },
            "lon": {
              "angle": {
                "radians": -0.07885639969290104,
                "degrees": -4.518138889999951,
                "percentage": -7.902025902955326,
                "degMinSec": {}
              },
              "radians": -0.07885639969290104,
              "degrees": -4.518138889999951,
              "degMinSec": {},
              "vrc": "E004.31.005.300",
              "nats": "0043105W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        },
        {
          "startPoint": {
            "lat": {
              "angle": {
                "radians": 0.9503732778008005,
                "degrees": 54.45237778,
                "percentage": 139.94863782854935,
                "degMinSec": {}
              },
              "radians": 0.9503732778008005,
              "degrees": 54.45237778,
              "degMinSec": {},
              "vrc": "N054.27.008.560",
              "nats": "542709N"
            },
            "lon": {
              "angle": {
                "radians": -0.09311016422740082,
                "degrees": -5.334819440000042,
                "percentage": -9.338017354418039,
                "degMinSec": {}
              },
              "radians": -0.09311016422740082,
              "degrees": -5.334819440000042,
              "degMinSec": {},
              "vrc": "E005.20.005.350",
              "nats": "0052005W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          },
          "endPoint": {
            "lat": {
              "angle": {
                "radians": 0.956292707344866,
                "degrees": 54.79153611,
                "percentage": 141.71458787308552,
                "degMinSec": {}
              },
              "radians": 0.956292707344866,
              "degrees": 54.79153611,
              "degMinSec": {},
              "vrc": "N054.47.029.530",
              "nats": "544730N"
            },
            "lon": {
              "angle": {
                "radians": -0.09788397919814429,
                "degrees": -5.608338890000012,
                "percentage": -9.819779969455576,
                "degMinSec": {}
              },
              "radians": -0.09788397919814429,
              "degrees": -5.608338890000012,
              "degMinSec": {},
              "vrc": "E005.36.030.020",
              "nats": "0053630W"
            },
            "alt": {
              "meters": 0,
              "feet": 0,
              "nauticalMiles": 0,
              "statuteMiles": 0
            }
          }
        }
      ],
      "alongTrackDistance_m": -1,
      "crossTrackDistance_m": -1,
      "requiredTrueCourse": -1,
      "turnRadius_m": -1
    },
    "autopilot": {
      "selectedHeading": 309,
      "selectedHeadingBearing": {
        "angle": {
          "radians": 5.3930673886624785,
          "degrees": 309,
          "percentage": -123.48971565350513,
          "degMinSec": {}
        },
        "radians": 5.3930673886624785,
        "degrees": 309
      },
      "selectedAltitude": 10000,
      "selectedAltitudeLength": {
        "meters": 3047.9999024640033,
        "feet": 10000,
        "nauticalMiles": 1.6457882842678204,
        "statuteMiles": 1.8939393333333354
      },
      "selectedVerticalSpeed": 0,
      "selectedVerticalSpeedVelocity": {
        "metersPerSecond": 0,
        "knots": 0,
        "feetPerMinute": 0
      },
      "selectedFpa": 0,
      "selectedFpaAngle": {
        "radians": 0,
        "degrees": 0,
        "percentage": 0,
        "degMinSec": {}
      },
      "selectedSpeedMode": "FMS",
      "selectedSpeedUnits": "KNOTS",
      "selectedSpeed": 250,
      "currentLateralMode": "HDG",
      "armedLateralModes": [
        "LNAV"
      ],
      "currentVerticalMode": "FLCH",
      "armedVerticalModes": [],
      "currentThrustMode": "SPEED",
      "armedThrustModes": [],
      "hdgKnobTurnDirection": "SHORTEST"
    },
    "connectionStatus": "WAITING",
    "aircraftType": "A320",
    "airlineCode": "JBU",
    "flightPhase": "IN_FLIGHT"
  }
]"#;

impl ApiLink {
    pub fn new(hostname: String, port: u16) -> Self {

        let sim_aircraft: Vec<SimAircraft> = serde_json::from_str(INPUT).unwrap();

        let (rta_tx, rta_rx) = mpsc::channel::<radar_to_ui::PacketType>();
        let (msg_tx, msg_rx) = mpsc::channel::<ImplMessage>();
        let thread_should_terminate = Arc::new(AtomicBool::new(false));
        
        let hostname = format_hostname(&hostname, port);
        
        let thread = api_worker(Arc::clone(&thread_should_terminate), msg_tx, rta_rx, hostname);


        ApiLink { thread_should_terminate, rta_tx, msg_rx, thread: Some(thread) }
    }
    pub fn poll(&mut self, max: usize) -> Vec<Message> {
        let mut count = 0;
        let mut vec = Vec::with_capacity(max);
        while let Ok(impl_message) = self.msg_rx.try_recv() {
            if count == max { break; }
            match impl_message {
                ImplMessage::Message(message) => {
                    vec.push(message);
                    count += 1;
                },
            }
        }
        vec
    }
    pub fn send(&self, packet: radar_to_ui::PacketType) {
        self.rta_tx.send(packet).ok();
    }
}

impl Drop for ApiLink {
    fn drop(&mut self) {
        self.thread_should_terminate.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}


fn api_worker(thread_should_terminate: Arc<AtomicBool>, msg_tx: Sender<ImplMessage>, rta_rx: Receiver<radar_to_ui::PacketType>, hostname: String) -> JoinHandle<()> {
    
    thread::spawn(move || {

        let hostname = hostname;
        let aircraft_data_endpoint = format!("{hostname}{AIRCRAFT_DATA_ENDPOINT}");
        let log_buffer_endpoint = format!("{hostname}{LOG_BUFFER_ENDPOINT}");
        let text_command_endpoint = format!("{hostname}{TEXT_COMMAND_ENDPOINT}");
        let client = ureq::AgentBuilder::new().timeout(API_REQUEST_TIMEOUT).build();
        let mut count = 0;
        loop {
            if thread_should_terminate.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(API_POLL_INTERVAL);

            // Get aircraft data
            
            if count == 0 {
                if let Some(data) =  client.get(&aircraft_data_endpoint).call().ok().and_then(|response| response.into_json::<Vec<SimAircraft>>().ok()).map(|vec| vec.into_iter().map(AircraftUpdate::from).collect::<Vec<_>>()) {
                    msg_tx.send(ImplMessage::Message(Message::AircraftDataUpdate(data))).ok();
                }
            }

            // Get log buffer
            if let Some(data) = client.get(&log_buffer_endpoint).call().ok().and_then(|response| response.into_json::<Vec<String>>().ok()) {
                for log_msg in data {
                    msg_tx.send(ImplMessage::Message(Message::LogMessage(log_msg))).ok();
                }
            }

            // Send any requests if there are any to send
            if let Ok(PacketType::ApiRequest(ApiRequestType::TextCommand(text_command_request))) = rta_rx.try_recv() {
                client.post(&text_command_endpoint).send_json(&text_command_request).ok();
            }
            count += 1;
            if count == 10 { count = 0};
        }

    })
}


#[derive(Debug, Clone)]
pub enum Message {
    AircraftDataUpdate(Vec<AircraftUpdate>),
    LogMessage(String),
}

#[derive(Debug)]
enum ImplMessage {
    Message(Message),
}

fn format_hostname(hostname: &str, port: u16) -> String {
    let hostname = hostname.trim_start_matches("http://");
    format!("http://{hostname}:{port}")
}

































#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraft {
    callsign: String,
    delay_ms: i32,
    sim_state: SimState,
    fms: Fms,
    position: SimAircraftPosition,
    autopilot: Autopilot,
    connection_status: ConnectionStatus,
}

impl From<SimAircraft> for common::aircraft_data::AircraftUpdate {
    fn from(value: SimAircraft) -> Self {
        let callsign = value.callsign;
        let position = common::position::Position::new_with_alt(value.position.latitude.degrees, value.position.longitude.degrees, value.position.indicated_altitude.feet);
        let heading_mag = value.position.magnetic_heading;
        let heading_true = value.position.true_heading;
        let track_mag = value.position.track_mag;
        let track_true = value.position.track_true;
        let pitch = value.position.pitch;
        let bank = value.position.bank;
        let indicated_airspeed = value.position.indicated_air_speed;
        let mach_number = value.position.mach_number;
        let ground_speed = value.position.ground_speed;
        let vertical_speed = value.position.vertical_speed;
        let wind_direction = value.position.wind_direction;
        let wind_speed = value.position.wind_speed;
        let on_ground = value.position.on_ground;
        let altimeter_setting_hpa = value.position.altimeter_setting;
        let autopilot = value.autopilot;
        let fms_string = value.fms.as_string;
        let fms_graphics = value.fms.fms_lines.into_iter().map(FmsGraphic::from).collect::<Vec<_>>();
        let sim_rate = value.sim_state.sim_rate;
        let is_paused = value.sim_state.paused;
        let connection_status = match value.connection_status {
            ConnectionStatus::Connected => common::aircraft_data::ConnectionStatus::Connected,
            ConnectionStatus::Disconnected => common::aircraft_data::ConnectionStatus::Disconnected,
            ConnectionStatus::Connecting => common::aircraft_data::ConnectionStatus::Connecting,
            ConnectionStatus::Waiting => common::aircraft_data::ConnectionStatus::Waiting(value.delay_ms),
        };

        common::aircraft_data::AircraftUpdate {
            callsign,
            data: common::aircraft_data::AircraftData {
                position,
                heading_mag: heading_mag.degrees,
                heading_true: heading_true.degrees,
                track_mag: track_mag.degrees,
                track_true: track_true.degrees,
                pitch: pitch.degrees,
                bank: bank.degrees,
                indicated_airspeed: indicated_airspeed.knots,
                mach_number,
                ground_speed: ground_speed.knots,
                vertical_speed: vertical_speed.feet_per_minute,
                wind_direction: wind_direction.degrees,
                wind_speed: wind_speed.knots,
                on_ground,
                altimeter_setting_hpa: altimeter_setting_hpa.hectopascals,
                autopilot,
                fms_string,
                fms_graphics,
                sim_rate,
                is_paused,
                connection_status
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimState {
    sim_rate: f32,
    paused: bool,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftPosition {
    latitude: SimAircraftAngle,
    longitude: SimAircraftAngle,
    #[serde(rename = "heading_Mag")]
    magnetic_heading: SimAircraftAngle,
    #[serde(rename = "heading_True")]
    true_heading: SimAircraftAngle,
    #[serde(rename = "track_Mag")]
    track_mag: SimAircraftAngle,
    #[serde(rename = "track_True")]
    track_true: SimAircraftAngle,
    bank: SimAircraftAngle,
    pitch: SimAircraftAngle,
    mach_number: f32,
    vertical_speed: SimAircraftSpeed,
    on_ground: bool,
    indicated_altitude: SimAircraftAltitude,
    indicated_air_speed: SimAircraftSpeed,
    ground_speed: SimAircraftSpeed,
    #[serde(rename = "altimeterSetting")]
    altimeter_setting: SimAircraftPressure,
    wind_direction: SimAircraftAngle,
    wind_speed: SimAircraftSpeed,
}


#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionStatus {
    Waiting,
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fms {
    pub as_string: String,
    pub fms_lines: Vec<SimAircraftFmsLine>,
}


#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftAngle {
    pub radians: f32,
    pub degrees: f32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftPressure {
    pub pascals: f32,
    pub hectopascals: f32,
    pub inches_of_mercury: f32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftAltitude {
    pub meters: f32,
    pub feet: f32,
    pub nautical_miles: f32,
    pub statute_miles: f32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftSpeed {
    pub meters_per_second: f32,
    pub knots: f32,
    pub feet_per_minute: f32,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftPoint {
    pub lat: SimAircraftAngle,
    pub lon: SimAircraftAngle,
    pub alt: SimAircraftAltitude
}



#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum SimAircraftFmsLine {
    #[serde(rename_all = "camelCase")]
    Arc  { start_point: SimAircraftPoint, end_point: SimAircraftPoint, center: SimAircraftPoint, #[serde(rename = "radius_m")] radius_m: f32, start_true_bearing: SimAircraftAngle, end_true_bearing: SimAircraftAngle, clockwise: bool },

    #[serde(rename_all = "camelCase")]
    Line { start_point: SimAircraftPoint, end_point: SimAircraftPoint },
}

impl From<SimAircraftFmsLine> for FmsGraphic {
    fn from(value: SimAircraftFmsLine) -> Self {
        match value {
            SimAircraftFmsLine::Line { start_point, end_point } => {
                FmsGraphic::Line(FmsLine { start: Position::new(start_point.lat.degrees, start_point.lon.degrees), end: Position::new(end_point.lat.degrees, end_point.lon.degrees) })
            },
            SimAircraftFmsLine::Arc { center, radius_m, start_true_bearing, end_true_bearing, clockwise, .. } => {
                FmsGraphic::Arc(
                    FmsArc {
                        state: FmsArcState::Uninitialised { centre: Position::new(center.lat.degrees, center.lon.degrees), radius_m, start_bearing_true: start_true_bearing.degrees, end_bearing_true: end_true_bearing.degrees, clockwise },
                    }
                )
            },
        }
    }
}