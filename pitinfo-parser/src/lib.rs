use lazy_static::lazy_static;
use regex::Regex;

#[derive(PartialEq, Debug)]
pub enum DayColor {
    Blue,
    White,
    Red,
}

#[derive(PartialEq, Debug)]
pub enum TariffOptionValue {
    Base,
    OffPeakHours,
    EJP,
    Tempo,
}

#[derive(PartialEq, Debug)]
pub enum HHPHCValue {
    A,
    C,
    D,
    E,
    Y,
}

#[derive(PartialEq, Debug)]
pub enum HourlyTarifPeriod {
    OffPeakHours,
    PeakHours,
}

#[derive(PartialEq, Debug)]
pub struct TarifPeriod {
    hour: HourlyTarifPeriod,
    day_color: Option<DayColor>,
}

#[derive(PartialEq, Debug)]
pub enum Message {
    ADCO,
    TariffOption(TariffOptionValue),
    Tomorrow(Option<DayColor>),
    InstantaneousPower { phase: u8, value: u8 },
    Index { period: TarifPeriod, value: u32 },
    ApparentPower { value: u16 },
    HHPHC(HHPHCValue),
    CurrentTariffPeriod(TarifPeriod)
}

pub fn parse_line(line: &str) -> Result<Option<Message>, String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            "^(ADCO|DEMAIN|OPTARIF|IINST[123]|BBRH[CP]J[BWR]|MOTDETAT|IMAX[123]|PPOT|PMAX|PAPP|ISOUSC|HHPHC|PTEC)\
        [ U+0009](.+)[ U+0009](.)$"
        )
        .unwrap();
    }
    let captures = RE.captures(line);

    if let Some(captures) = captures {
        let code = captures.get(1).unwrap().as_str();
        let data = captures.get(2).unwrap().as_str();
        let control = captures.get(3).unwrap().as_str();

        return match code {
            "ADCO" => Ok(Some(Message::ADCO)),
            "BBRHCJB" | "BBRHCJW" | "BBRHCJR" | "BBRHPJB" | "BBRHPJW" | "BBRHPJR" => {
                match data.parse::<u32>() {
                    Ok(value) => Ok(Some(Message::Index {
                        period: parse_period(&code[3..])?,
                        value: value
                    })),
                    Err(_e) => Err(format!("Unable to parse {} data: '{}'", code, data))
                }
            },
            "PTEC" => {
                match data {
                    "HCJB" => Ok(Some(Message::CurrentTariffPeriod(TarifPeriod { 
                        hour: HourlyTarifPeriod::OffPeakHours,
                        day_color: Some(DayColor::Blue)
                    } ))),
                    "HCJW" => Ok(Some(Message::CurrentTariffPeriod(TarifPeriod { 
                        hour: HourlyTarifPeriod::OffPeakHours,
                        day_color: Some(DayColor::White)
                    } ))),
                    "HCJR" => Ok(Some(Message::CurrentTariffPeriod(TarifPeriod { 
                        hour: HourlyTarifPeriod::OffPeakHours,
                        day_color: Some(DayColor::Red)
                    } ))),
                    "HPJB" => Ok(Some(Message::CurrentTariffPeriod(TarifPeriod { 
                        hour: HourlyTarifPeriod::PeakHours,
                        day_color: Some(DayColor::Blue)
                    } ))),
                    "HPJW" => Ok(Some(Message::CurrentTariffPeriod(TarifPeriod { 
                        hour: HourlyTarifPeriod::PeakHours,
                        day_color: Some(DayColor::White)
                    } ))),
                    "HPJR" => Ok(Some(Message::CurrentTariffPeriod(TarifPeriod { 
                        hour: HourlyTarifPeriod::PeakHours,
                        day_color: Some(DayColor::Red)
                    } ))),
                    _ => Err(format!("Unable to parse PTEC data: '{}'", data)),
                    
                }
            }
            "IINST1" | "IINST2" | "IINST3" => match data.parse::<u8>() {
                Ok(level) => Ok(Some(Message::InstantaneousPower {
                    phase: code.chars().nth(5).unwrap().to_digit(10).unwrap() as u8,
                    value: level,
                })),
                Err(_e) => Err(format!("Unable to parse {} data: '{}'", code, data)),
            },
            "OPTARIF" => match data {
                "BASE" => Ok(Some(Message::TariffOption(TariffOptionValue::Base))),
                "HC.." => Ok(Some(Message::TariffOption(TariffOptionValue::OffPeakHours))),
                "EJP." => Ok(Some(Message::TariffOption(TariffOptionValue::EJP))),
                _ => {
                    if data.starts_with("BBR") {
                        Ok(Some(Message::TariffOption(TariffOptionValue::Tempo)))
                    } else {
                        Err(format!("Unable to parse OPTARIF data: '{}'", data))
                    }
                }
            },
            "DEMAIN" => match data {
                "----" => Ok(Some(Message::Tomorrow(None))),
                "BLEU" => Ok(Some(Message::Tomorrow(Some(DayColor::Blue)))),
                "BLAN" => Ok(Some(Message::Tomorrow(Some(DayColor::White)))),
                "ROUG" => Ok(Some(Message::Tomorrow(Some(DayColor::Red)))),
                _ => Err(format!("Unable to parse DEMAIN data: '{}'", data)),
            },
            "PAPP" => match data.parse::<u16>() {
                Ok(value) => Ok(Some(Message::ApparentPower { value: value })),
                Err(_) => Err(format!("Unable to parse PAPP data: '{}'", data)),
            },
            "HHPHC" => match data {
                "A" => Ok(Some(Message::HHPHC(HHPHCValue::A))),
                "C" => Ok(Some(Message::HHPHC(HHPHCValue::C))),
                "D" => Ok(Some(Message::HHPHC(HHPHCValue::D))),
                "E" => Ok(Some(Message::HHPHC(HHPHCValue::E))),
                "Y" => Ok(Some(Message::HHPHC(HHPHCValue::Y))),
                _ => Err(format!("Unable to parse HHPHC data: '{}'", data)),
            },
            // The following codes are ignored
            "MOTDETAT" | "IMAX1" | "IMAX2" | "IMAX3" | "PPOT" | "PMAX" | "ISOUSC" => Ok(None),
            _ => panic!("Matching a code that is not recognized should never happen"),
        };
    }
    Err(String::from(format!("Unrecognized line: '{}'", line)))
}

fn parse_period(code: &str) -> Result<TarifPeriod, String> {
    // HCJB

    let hour = code.chars().nth(1).unwrap();
    let hour = if hour == 'C' {
        HourlyTarifPeriod::OffPeakHours
    } else if hour == 'P' {
        HourlyTarifPeriod::PeakHours
    } else {
        return Err(format!("Unable to parse hourly period from {}", code));
    };

    let day = code.chars().nth(3).unwrap();
    let day = match day {
        'B' => DayColor::Blue,
        'W' => DayColor::White,
        'R' => DayColor::Red,
        _ => return Err(format!("Unable to parse day color period from {}", code)),
    };

    Ok(TarifPeriod {
        hour: hour,
        day_color: Some(day),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_adco() {
        assert_eq!(parse_line("ADCO 020830022493 8"), Ok(Some(Message::ADCO)));
    }

    #[test]
    fn parse_tomorrow_undefined() {
        assert_eq!(
            parse_line("DEMAIN ---- \""),
            Ok(Some(Message::Tomorrow(None)))
        );
    }

    #[test]
    fn parse_tomorrow_blue() {
        // TODO: correct control char
        assert_eq!(
            parse_line("DEMAIN BLEU +"),
            Ok(Some(Message::Tomorrow(Some(DayColor::Blue))))
        );
    }

    #[test]
    fn parse_tomorrow_white() {
        // TODO: correct control char
        assert_eq!(
            parse_line("DEMAIN BLAN +"),
            Ok(Some(Message::Tomorrow(Some(DayColor::White))))
        );
    }

    #[test]
    fn parse_tomorrow_red() {
        assert_eq!(
            parse_line("DEMAIN ROUG +"),
            Ok(Some(Message::Tomorrow(Some(DayColor::Red))))
        );
    }

    #[test]
    fn parse_opttarif_base() {
        // TODO: correct control char
        assert_eq!(
            parse_line("OPTARIF BASE S"),
            Ok(Some(Message::TariffOption(TariffOptionValue::Base)))
        );
    }

    #[test]
    fn parse_opttarif_heures_creuses() {
        // TODO: correct control char
        assert_eq!(
            parse_line("OPTARIF HC.. S"),
            Ok(Some(Message::TariffOption(TariffOptionValue::OffPeakHours)))
        );
    }

    #[test]
    fn parse_opttarif_ejp() {
        // TODO: correct control char
        assert_eq!(
            parse_line("OPTARIF EJP. S"),
            Ok(Some(Message::TariffOption(TariffOptionValue::EJP)))
        );
    }

    #[test]
    fn parse_opttarif_bbr() {
        assert_eq!(
            parse_line("OPTARIF BBR( S"),
            Ok(Some(Message::TariffOption(TariffOptionValue::Tempo)))
        );
    }

    #[test]
    fn parse_opttarif_bad_data() {
        // TODO: correct control char
        assert_eq!(
            parse_line("OPTARIF ABCD S"),
            Err(String::from("Unable to parse OPTARIF data: 'ABCD'"))
        );
    }

    #[test]
    fn parse_iinstx() {
        // TODO: correct control char
        assert_eq!(
            parse_line("IINST1 0 S"),
            Ok(Some(Message::InstantaneousPower { phase: 1, value: 0 }))
        );
        assert_eq!(
            parse_line("IINST2 0 S"),
            Ok(Some(Message::InstantaneousPower { phase: 2, value: 0 }))
        );
        assert_eq!(
            parse_line("IINST3 0 S"),
            Ok(Some(Message::InstantaneousPower { phase: 3, value: 0 }))
        );
        assert_eq!(
            parse_line("IINST1 1 S"),
            Ok(Some(Message::InstantaneousPower { phase: 1, value: 1 }))
        );
        assert_eq!(
            parse_line("IINST2 1 S"),
            Ok(Some(Message::InstantaneousPower { phase: 2, value: 1 }))
        );
        assert_eq!(
            parse_line("IINST3 1 S"),
            Ok(Some(Message::InstantaneousPower { phase: 3, value: 1 }))
        );
        assert_eq!(
            parse_line("IINST1 33 S"),
            Ok(Some(Message::InstantaneousPower {
                phase: 1,
                value: 33
            }))
        );
        assert_eq!(
            parse_line("IINST2 33 S"),
            Ok(Some(Message::InstantaneousPower {
                phase: 2,
                value: 33
            }))
        );
        assert_eq!(
            parse_line("IINST3 33 S"),
            Ok(Some(Message::InstantaneousPower {
                phase: 3,
                value: 33
            }))
        );
        assert_eq!(
            parse_line("IINST1 A S"),
            Err(String::from("Unable to parse IINST1 data: 'A'"))
        );
        assert_eq!(
            parse_line("IINST2 A S"),
            Err(String::from("Unable to parse IINST2 data: 'A'"))
        );
        assert_eq!(
            parse_line("IINST3 A S"),
            Err(String::from("Unable to parse IINST3 data: 'A'"))
        );
    }

    #[test]
    fn parse_bbrhcjc() {
        assert_eq!(
            parse_line("BBRHCJB 023916830 ="), // control OK
            Ok(Some(Message::Index {
                period: TarifPeriod {
                    hour: HourlyTarifPeriod::OffPeakHours,
                    day_color: Some(DayColor::Blue)
                },
                value: 23916830 
            }))
        );
        assert_eq!(
            parse_line("BBRHCJB a -"),
            Err(String::from("Unable to parse BBRHCJB data: 'a'"))
        );
    }

    #[test]
    fn parse_bbrhcjw() {
        assert_eq!(
            parse_line("BBRHCJW 023916830 ="), // control OK
            Ok(Some(Message::Index {
                period: TarifPeriod {
                    hour: HourlyTarifPeriod::OffPeakHours,
                    day_color: Some(DayColor::White)
                },
                value: 23916830 
            }))
        );
        assert_eq!(
            parse_line("BBRHCJW a -"),
            Err(String::from("Unable to parse BBRHCJW data: 'a'"))
        );
    }

    #[test]
    fn parse_bbrhcjr() {
        assert_eq!(
            parse_line("BBRHCJR 023916830 ="), // control OK
            Ok(Some(Message::Index {
                period: TarifPeriod {
                    hour: HourlyTarifPeriod::OffPeakHours,
                    day_color: Some(DayColor::Red)
                },
                value: 23916830 
            }))
        );
        assert_eq!(
            parse_line("BBRHCJR a -"),
            Err(String::from("Unable to parse BBRHCJR data: 'a'"))
        );
    }

    #[test]
    fn parse_bbrhpjb() {
        assert_eq!(
            parse_line("BBRHPJB 023916830 ="), // control OK
            Ok(Some(Message::Index {
                period: TarifPeriod {
                    hour: HourlyTarifPeriod::PeakHours,
                    day_color: Some(DayColor::Blue)
                },
                value: 23916830 
            }))
        );
        assert_eq!(
            parse_line("BBRHPJB a -"),
            Err(String::from("Unable to parse BBRHPJB data: 'a'"))
        );
    }

    #[test]
    fn parse_bbrhpjw() {
        assert_eq!(
            parse_line("BBRHPJW 023916830 ="), // control OK
            Ok(Some(Message::Index {
                period: TarifPeriod {
                    hour: HourlyTarifPeriod::PeakHours,
                    day_color: Some(DayColor::White)
                },
                value: 23916830 
            }))
        );
        assert_eq!(
            parse_line("BBRHPJW a -"),
            Err(String::from("Unable to parse BBRHPJW data: 'a'"))
        );
    }

    #[test]
    fn parse_bbrhpjr() {
        assert_eq!(
            parse_line("BBRHPJR 023916830 ="), // control OK
            Ok(Some(Message::Index {
                period: TarifPeriod {
                    hour: HourlyTarifPeriod::PeakHours,
                    day_color: Some(DayColor::Red)
                },
                value: 23916830 
            }))
        );
        assert_eq!(
            parse_line("BBRHPJR a -"),
            Err(String::from("Unable to parse BBRHPJR data: 'a'"))
        );
    }

    #[test]
    fn parse_papp() {
        assert_eq!(
            parse_line("PAPP 00803 ,"), // control OK
            Ok(Some(Message::ApparentPower { value: 803 }))
        );
        assert_eq!(
            parse_line("PAPP 00813 -"), // control OK
            Ok(Some(Message::ApparentPower { value: 813 }))
        );
        assert_eq!(
            parse_line("PAPP a -"),
            Err(String::from("Unable to parse PAPP data: 'a'"))
        );
    }

    #[test]
    fn parse_hhphc() {
        // TODO: correct control char
        assert_eq!(
            parse_line("HHPHC A D"),
            Ok(Some(Message::HHPHC(HHPHCValue::A)))
        );
        assert_eq!(
            parse_line("HHPHC C D"),
            Ok(Some(Message::HHPHC(HHPHCValue::C)))
        );
        assert_eq!(
            parse_line("HHPHC D D"),
            Ok(Some(Message::HHPHC(HHPHCValue::D)))
        );
        assert_eq!(
            parse_line("HHPHC E D"),
            Ok(Some(Message::HHPHC(HHPHCValue::E)))
        );
        assert_eq!(
            parse_line("HHPHC Y D"), // control is OK
            Ok(Some(Message::HHPHC(HHPHCValue::Y)))
        );
        assert_eq!(
            parse_line("HHPHC X D"),
            Err(String::from("Unable to parse HHPHC data: 'X'"))
        );
    }

    #[test]
    fn parse_ptec() {
        
        assert_eq!(
            parse_line("PTEC HCJR S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::Red)
            })))
        );
        assert_eq!(
            parse_line("PTEC HCJR S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::Red)
            })))
        );
        assert_eq!(
            parse_line("PTEC HCJB S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::Blue)
            })))
        );
        assert_eq!(
            parse_line("PTEC HCJW S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::White)
            })))
        );
        assert_eq!(
            parse_line("PTEC HCJR S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::Red)
            })))
        );
        assert_eq!(
            parse_line("PTEC HPJB S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::PeakHours,
                day_color: Some(DayColor::Blue)
            })))
        );
        assert_eq!(
            parse_line("PTEC HPJW S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::PeakHours,
                day_color: Some(DayColor::White)
            })))
        );
        assert_eq!(
            parse_line("PTEC HPJR S"), // control is OK
            Ok(Some(Message::CurrentTariffPeriod(TarifPeriod {
                hour: HourlyTarifPeriod::PeakHours,
                day_color: Some(DayColor::Red)
            })))
        );
        assert_eq!(
            parse_line("PTEC XXXX S"),
            Err(String::from("Unable to parse PTEC data: 'XXXX'"))
        );        
    }

    /*
     * Un recognized lines
     */

    #[test]
    fn parse_iinst4() {
        // TODO: correct control char
        assert_eq!(
            parse_line("IINST4 3 S"),
            Err(String::from("Unrecognized line: 'IINST4 3 S'"))
        );
    }

    #[test]
    fn parse_unknown_code() {
        assert_eq!(
            parse_line("XXX AAA"),
            Err(String::from("Unrecognized line: 'XXX AAA'"))
        );
    }

    #[test]
    fn parse_code_without_value() {
        assert_eq!(
            parse_line("XXX"),
            Err(String::from("Unrecognized line: 'XXX'"))
        );
    }

    /**
     * Parse periods
     */

    #[test]
    fn parse_period_error() {
        assert_eq!(
            parse_period("HAJB"),
            Err(String::from("Unable to parse hourly period from HAJB"))
        );
        assert_eq!(
            parse_period("HCJT"),
            Err(String::from(
                "Unable to parse day color period from HCJT"
            ))
        );
    }

    #[test]
    fn parse_period_ok() {
        assert_eq!(
            parse_period("HCJB"),
            Ok(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::Blue)
            })
        );
        assert_eq!(
            parse_period("HCJW"),
            Ok(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::White)
            })
        );
        assert_eq!(
            parse_period("HCJR"),
            Ok(TarifPeriod {
                hour: HourlyTarifPeriod::OffPeakHours,
                day_color: Some(DayColor::Red)
            })
        );
        assert_eq!(
            parse_period("HPJB"),
            Ok(TarifPeriod {
                hour: HourlyTarifPeriod::PeakHours,
                day_color: Some(DayColor::Blue)
            })
        );
        assert_eq!(
            parse_period("HPJW"),
            Ok(TarifPeriod {
                hour: HourlyTarifPeriod::PeakHours,
                day_color: Some(DayColor::White)
            })
        );
        assert_eq!(
            parse_period("HPJR"),
            Ok(TarifPeriod {
                hour: HourlyTarifPeriod::PeakHours,
                day_color: Some(DayColor::Red)
            })
        );
    }
}

/* Sample data:

ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023916830 =
BBRHPJB 045909975 Z
BBRHCJW 007127242 K
BBRHPJW 013332976 !
BBRHCJR 004353593 M
BBRHPJR 007659709 %
PTEC HPJR
DEMAIN ---- "
IINST1 009 Q
IINST2 007 P
IINST3 009 S
IMAX1 031 4
IMAX2 034 8
IMAX3 029 =
PMAX 13190 4
PAPP 05998 @
HHPHC Y D
MOTDETAT 000000 B
PPOT 00 #

ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023916830 =
BBRHPJB 045909975 Z
BBRHCJW 007127242 K
BBRHPJW 013332976 !
BBRHCJR 004353593 M
BBRHPJR 007659709 %
PTEC HPJR
DEMAIN ---- "
IINST1 009 Q
IINST2 007 P
IINST3 009 S
IMAX1 031 4
IMAX2 034 8
IMAX3 029 =
PMAX 13190 4
PAPP 05998 @
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
BBRHCJR 004270634 G
BBRHPJR 007507586
PTEC HPJR
DEMAIN ---- "
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
DEMAIN ROUG +
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

ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023916830 =
BBRHPJB 045909975 Z
BBRHCJW 007127242 K
BBRHPJW 013332976 !
BBRHCJR 004339153 I
BBRHPJR 007648380 ^
PTEC HCJR S
DEMAIN ROUG +
IINST1 007 O
IINST2 006 O
IINST3 008 R
IMAX1 031 4
IMAX2 034 8
IMAX3 029 =
PMAX 13190 4
PAPP 05195 5
HHPHC Y D
MOTDETAT 000000 B
PPOT 00 #

ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023916830 =
BBRHPJB 045909975 Z
BBRHCJW 007127242 K
BBRHPJW 013332976 !
BBRHCJR 004357

ADCO 020830022493 8
OPTARIF BBR( S
ISOUSC 30 9
BBRHCJB 023916830 =
BBRHPJB 045940890 Q
BBRHCJW 007161874 T
BBRHPJW 013397921 "
BBRHCJR 004372269 N
BBRHPJR 007686015 [
PTEC HPJB P
DEMAIN BLAN K
IINST1 007 O
IINST2 006 O
IINST3 008 R
IMAX1 031 4
IMAX2 034 8
IMAX3 029 =
PMAX 13190 4
PAPP 04881 6
HHPHC Y D
MOTDETAT 000000 B
PPOT 00 #

*/
