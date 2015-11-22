use std::str;
use nom;
use nom::IResult;

/// Try to parse a version
///
/// If there's an error, then you just get (). for now.
pub fn try_parse(i: &[u8]) -> Result<super::Version, String> {
    match version(i) {
        IResult::Done(rest, version) => {
            if rest.len() > 0 {
                let err = format!("Failed with unparsed input: '{}'",
                                  String::from_utf8(rest.to_vec()).unwrap());
                Err(err)
            } else{
                Ok(version)
            }
        },
        _ => Err("Parse error".to_string()),
    }
}

/// parse a u32
fn number(i: &[u8]) -> IResult<&[u8], u32> {
    map_res!(i,
             nom::digit,
             |d| str::FromStr::from_str(str::from_utf8(d).unwrap()))
}

/// Parse an alphanumeric or a dot ("[0-9A-Za-z.]" in regex)
fn ascii_or_hyphen(chr: u8) -> bool {
    // dot
    chr == 46 ||
    // 0-9
    (chr >= 48 && chr <= 57) ||
    // A-Z
    (chr >= 65 && chr <= 90) ||
    // a-z
    (chr >= 97 && chr <= 122)
}

/// Parse a word
fn word(i: &[u8]) -> IResult<&[u8], String> {
    map_res!(i,
             take_while!(ascii_or_hyphen),
             |d: &[u8]|
                 match d.len() {
                     0 => Err("Expected 1 or more characters"),
                     _ => Ok(String::from_utf8(d.to_vec()).unwrap()),
                 }
             )
}

/// parse a . and then a u32
named!(dot_number<&[u8], u32>, preceded!(char!('.'), number));

named!(pre<&[u8], Option<String> >,   opt!(complete!(preceded!(tag!("-"), word))));
named!(build<&[u8], Option<String> >, opt!(complete!(preceded!(tag!("+"), word))));

named!(extras<&[u8], (Option<String>, Option<String>) >, chain!(
        pre: pre ~
        build: build,
        || { (pre, build) }
));

/// parse a version
///
/// A version is currently:
///
/// - a major version number
/// - optionally followed by a dot and a minor version number
/// - optionally followed by a dot and a patch version number
///
/// If some of the versions aren't present, gives a zero.
named!(version<&[u8], super::Version>, chain!(
        major: number ~
        minor: dot_number ~
        patch: dot_number ~
        extras: extras,
        || {
            super::Version {
                major: major,
                minor: minor,
                patch: patch,
                pre: extras.0.clone(),
                build: extras.1.clone(),
            }
        }
));


#[cfg(test)]
mod tests {
    use super::number;
    use super::dot_number;
    use super::version;
    use Version;

    fn done<T>(x: T) -> ::nom::IResult<&'static [u8], T> {
        ::nom::IResult::Done(&[][..], x)
    }

    #[test]
    fn parse_number() {
        let v = "10".as_bytes();

        assert_eq!(number(v), done(10));
    }

    #[test]
    fn parse_dot_number() {
        let v = ".10".as_bytes();

        assert_eq!(dot_number(v), done(10));
    }

    #[test]
    fn parse_version() {
        let v1 = "10.11.12".as_bytes();
        let v2 = Version {
            major: 10,
            minor: 11,
            patch: 12,
            pre: None,
            build: None,
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_pre_basic() {
        let v1 = "1.0.0-alpha".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Some(String::from("alpha")),
            build: None,
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_pre_dot() {
        let v1 = "1.0.0-alpha.1".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Some(String::from("alpha.1")),
            build: None,
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_build_basic() {
        let v1 = "1.0.0-alpha+001".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Some(String::from("alpha")),
            build: Some(String::from("001")),
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_build_no_pre() {
        let v1 = "1.0.0+001".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: None,
            build: Some(String::from("001")),
        };

        assert_eq!(version(v1), done(v2));
    }
}
