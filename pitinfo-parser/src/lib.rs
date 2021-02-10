use lazy_static::lazy_static;
use regex::Regex;


#[derive(PartialEq, Debug)]
pub enum DayColor {
    Blue,
    White,
    Red
}

#[derive(PartialEq, Debug)]
pub enum TariffOptionValue {
    Base,
    OffPeakHours,
    EJP,
    Tempo,
}

#[derive(PartialEq, Debug)]
pub enum HourlyTarifPeriod {
    OffPeakHours,
    PeakHours
}

#[derive(PartialEq, Debug)]
pub struct TarifPeriod {
    hour: HourlyTarifPeriod,
    day_color: Option<DayColor>
}

#[derive(PartialEq, Debug)]
pub enum Message {
    ADCO,
    TariffOption(TariffOptionValue),
    Tomorrow(Option<DayColor>),
    InstantaneousPower{
        phase: u8, 
        value: u8},
    Index {
        period: TarifPeriod,
        value: f32
    }
}

pub fn parse_line(line: &str) -> Result<Message, String> {
    lazy_static! {
        static ref RE: Regex = Regex::new("^(ADCO|Tomorrow|OPTARIF|IINST[123]|BBRH[CP]J[BWR])\
        [ U+0009](.+)[ U+0009](.)$").unwrap();
    }
    let captures = RE.captures(line);

    if let Some(captures) = captures {
        let code = captures.get(1).unwrap().as_str();
        let data = captures.get(2).unwrap().as_str();
        let control = captures.get(3).unwrap().as_str();

        return match code {
            "ADCO"   => Ok(Message::ADCO),
            "BBRHCJB"|"BBRHCJW"|"BBRHCJR"|"BBRHPJB"|"BBRHPJW"|"BBRHPJR" => {
                Ok(Message::Index{
                    period: parse_period(code)?,
                    value: 0.0
                })
            }
            "IINST1"|"IINST2"|"IINST3" => {
                match data.parse::<u8>() {
                    Ok(level) => Ok(Message::InstantaneousPower { 
                        phase: code.chars().nth(5).unwrap().to_digit(10).unwrap() as u8, 
                        value: level
                    }),
                    Err(_e) => Err(format!("Unable to parse {} data: '{}'", code, data)),
                }
            }
            "OPTARIF" => {
                match data {
                    "BASE" => Ok(Message::TariffOption(TariffOptionValue::Base)),
                    "HC.." => Ok(Message::TariffOption(TariffOptionValue::OffPeakHours)),
                    "EJP." => Ok(Message::TariffOption(TariffOptionValue::EJP)),
                    _ => {
                        if data.starts_with("BBR") {
                            Ok(Message::TariffOption(TariffOptionValue::Tempo))
                        } else {
                            Err(format!("Unrecognized OPTARIF data: '{}'", data))
                        }
                    }
                }
            },
            "Tomorrow" => {
                match data {
                    "----" => Ok(Message::Tomorrow(None)),
                    "Blue" => Ok(Message::Tomorrow(Some(DayColor::Blue))),
                    "BLAN" => Ok(Message::Tomorrow(Some(DayColor::White))),
                    "ROUG" => Ok(Message::Tomorrow(Some(DayColor::Red))),
                    _ => Err(format!("Unrecognized Tomorrow data: '{}'", data)),
                }
            },
            _ => panic!("Matching a code that is not recognized should never happen"),
        };   
    }
    Err(String::from(format!("Unrecognized line: '{}'", line)))
}


fn parse_period(code: &str) -> Result<TarifPeriod, String> {
    // BBRHCJB

    let hour = code.chars().nth(4).unwrap();
    let hour = if hour == 'C' {
        HourlyTarifPeriod::OffPeakHours
    } else if hour == 'P' {
        HourlyTarifPeriod::PeakHours
    } else {
        return Err(format!("Unable to parse hourly period from {}", code));
    };

    let day = code.chars().nth(6).unwrap();
    let day = match day {
        'B' => DayColor::Blue,
        'W' => DayColor::White,
        'R' => DayColor::Red,
        _ => { return Err(format!("Unable to parse day color period from {}", code))}
    };

    Ok(TarifPeriod {
        hour: hour,
        day_color: Some(day)
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_adco() {
        assert_eq!(parse_line("ADCO 020830022493 8"), Ok(Message::ADCO));
    }

    #[test]
    fn parse_tomorrow_undefined() {
        assert_eq!(parse_line("Tomorrow ---- \""), Ok(Message::Tomorrow(None)));
    }

    #[test]
    fn parse_tomorrow_blue() {
        // TODO: correct control char
        assert_eq!(parse_line("Tomorrow Blue +"), Ok(Message::Tomorrow(Some(DayColor::Blue))));
    }

    #[test]
    fn parse_tomorrow_white() {
        // TODO: correct control char
        assert_eq!(parse_line("Tomorrow BLAN +"), Ok(Message::Tomorrow(Some(DayColor::White))));
    }

    #[test]
    fn parse_tomorrow_red() {
        assert_eq!(parse_line("Tomorrow ROUG +"), Ok(Message::Tomorrow(Some(DayColor::Red))));
    }

    #[test]
    fn parse_opttarif_base() {
        // TODO: correct control char
        assert_eq!(parse_line("OPTARIF BASE S"), Ok(Message::TariffOption(TariffOptionValue::Base)));        
    }

    #[test]
    fn parse_opttarif_heures_creuses() {
        // TODO: correct control char
        assert_eq!(parse_line("OPTARIF HC.. S"), Ok(Message::TariffOption(TariffOptionValue::OffPeakHours)));        
    }

    #[test]
    fn parse_opttarif_ejp() {
        // TODO: correct control char
        assert_eq!(parse_line("OPTARIF EJP. S"), Ok(Message::TariffOption(TariffOptionValue::EJP)));        
    }

    #[test]
    fn parse_opttarif_bbr() {
        assert_eq!(parse_line("OPTARIF BBR( S"), Ok(Message::TariffOption(TariffOptionValue::Tempo)));        
    }

    #[test]
    fn parse_opttarif_bad_data() {
        // TODO: correct control char
        assert_eq!(parse_line("OPTARIF ABCD S"), Err(String::from("Unrecognized OPTARIF data: 'ABCD'")));        
    }

    #[test]
    fn parse_iinstx() {
        // TODO: correct control char
        assert_eq!(parse_line("IINST1 0 S"), Ok(Message::InstantaneousPower{ phase: 1, value: 0}));        
        assert_eq!(parse_line("IINST2 0 S"), Ok(Message::InstantaneousPower{ phase: 2, value: 0}));        
        assert_eq!(parse_line("IINST3 0 S"), Ok(Message::InstantaneousPower{ phase: 3, value: 0}));        
        assert_eq!(parse_line("IINST1 1 S"), Ok(Message::InstantaneousPower{ phase: 1, value: 1}));        
        assert_eq!(parse_line("IINST2 1 S"), Ok(Message::InstantaneousPower{ phase: 2, value: 1}));        
        assert_eq!(parse_line("IINST3 1 S"), Ok(Message::InstantaneousPower{ phase: 3, value: 1}));        
        assert_eq!(parse_line("IINST1 33 S"), Ok(Message::InstantaneousPower{ phase: 1, value: 33}));        
        assert_eq!(parse_line("IINST2 33 S"), Ok(Message::InstantaneousPower{ phase: 2, value: 33}));        
        assert_eq!(parse_line("IINST3 33 S"), Ok(Message::InstantaneousPower{ phase: 3, value: 33}));        
        assert_eq!(parse_line("IINST1 A S"), Err(String::from("Unable to parse IINST1 data: 'A'")));        
        assert_eq!(parse_line("IINST2 A S"), Err(String::from("Unable to parse IINST2 data: 'A'")));        
        assert_eq!(parse_line("IINST3 A S"), Err(String::from("Unable to parse IINST3 data: 'A'")));        
    }

    /*
     * Un recognized lines
     */

    #[test]
    fn parse_iinst4() {
        // TODO: correct control char
        assert_eq!(parse_line("IINST4 3 S"), Err(String::from("Unrecognized line: 'IINST4 3 S'")));
    }

    #[test]
    fn parse_unknown_code() {
        assert_eq!(parse_line("XXX AAA"), Err(String::from("Unrecognized line: 'XXX AAA'")));
    }

    #[test]
    fn parse_code_without_value() {
        assert_eq!(parse_line("XXX"), Err(String::from("Unrecognized line: 'XXX'")));
    }

    /**
     * Parse periods
     */

    #[test]
    fn parse_period_error() {
        assert_eq!(parse_period("BBRHAJB"), Err(String::from("Unable to parse hourly period from BBRHAJB")));
        assert_eq!(parse_period("BBRHCJT"), Err(String::from("Unable to parse day color period from BBRHCJT")));
    }

    #[test]
    fn parse_period_ok() {
        assert_eq!(parse_period("BBRHCJB"), Ok(TarifPeriod { 
            hour: HourlyTarifPeriod::OffPeakHours, 
            day_color: Some(DayColor::Blue)
        }));
        assert_eq!(parse_period("BBRHCJW"), Ok(TarifPeriod { 
            hour: HourlyTarifPeriod::OffPeakHours, 
            day_color: Some(DayColor::White)
        }));
        assert_eq!(parse_period("BBRHCJR"), Ok(TarifPeriod { 
            hour: HourlyTarifPeriod::OffPeakHours, 
            day_color: Some(DayColor::Red)
        }));
        assert_eq!(parse_period("BBRHPJB"), Ok(TarifPeriod { 
            hour: HourlyTarifPeriod::PeakHours, 
            day_color: Some(DayColor::Blue)
        }));
        assert_eq!(parse_period("BBRHPJW"), Ok(TarifPeriod { 
            hour: HourlyTarifPeriod::PeakHours, 
            day_color: Some(DayColor::White)
        }));
        assert_eq!(parse_period("BBRHPJR"), Ok(TarifPeriod { 
            hour: HourlyTarifPeriod::PeakHours, 
            day_color: Some(DayColor::Red)
        }));
    }
}

/* Sample data:

ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023823656 @
BBRHPJB 045762037 L
BBRHCJW 007092953 U
BBRHPJW 013282053 W
BBRHCJR 004270634 G
BBRHPJR 007507586
PTEC HPJR
Tomorrow ---- "
IINST1 008 P
IINST2 006 O
IINST3 008 R
IMAX1 031 4
IMAX2 034 8
IMAX3 029 =
PMAX 13190 4
PAPP 05355 3
HHPHC Y D
MOTDETAT 000000 B
PPOT 00 #


ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023823656 @
BBRHPJB 045762037 L
BBRHCJW 007092953 U
BBRHPJW 013282053 W
BBRHCJR 004284807 N
BBRHPJR 007534260 U
PTEC HCJR S
Tomorrow ROUG +
IINST1 001 I
IINST2 000 I
IINST3 001 K
IMAX1 031 4
IMAX2 034 8
IMAX3 029 =
PMAX 13190 4
PAPP 00549 3
HHPHC Y D
MOTDETAT 000000 B
PPOT 00 #

*/