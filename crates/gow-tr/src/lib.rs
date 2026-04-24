use clap::Parser;
use std::ffi::OsString;
use std::io::{self, Read, Write};

#[derive(Parser)]
#[command(name = "tr", about = "GNU tr — Windows port.", version)]
struct Args {
    #[arg(short = 'c', short_alias = 'C', long = "complement", help = "use the complement of SET1")]
    complement: bool,

    #[arg(short = 'd', long = "delete", help = "delete characters in SET1, do not translate")]
    delete: bool,

    #[arg(short = 's', long = "squeeze-repeats", help = "replace each sequence of a repeated character that is listed in the last specified SET, with a single occurrence of that character")]
    squeeze: bool,

    #[arg(short = 't', long = "truncate-set1", help = "first truncate SET1 to length of SET2")]
    truncate: bool,

    #[arg(help = "SET1")]
    set1: String,

    #[arg(help = "SET2")]
    set2: Option<String>,
}

pub fn uumain<I: IntoIterator<Item = OsString>>(args: I) -> i32 {
    gow_core::init();

    let args = match Args::try_parse_from(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    if args.delete && args.set2.is_some() && !args.squeeze {
        eprintln!("tr: extra operand '{}'", args.set2.unwrap());
        eprintln!("Only one string may be given when deleting without squeezing repeats.");
        return 1;
    }

    if !args.delete && !args.squeeze && args.set2.is_none() {
        eprintln!("tr: missing operand after '{}'", args.set1);
        eprintln!("Two strings must be given when translating.");
        return 1;
    }

    // Expand sets
    let mut s1 = expand_set(&args.set1);
    let s2 = args.set2.as_ref().map(|s| expand_set(s)).unwrap_or_default();

    if args.complement {
        let mut complement_set = Vec::new();
        let mut is_in_s1 = [false; 256];
        for &b in &s1 {
            is_in_s1[b as usize] = true;
        }
        for b in 0..256 {
            if !is_in_s1[b] {
                complement_set.push(b as u8);
            }
        }
        s1 = complement_set;
    }

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut buf = [0u8; 8192];
    let mut last_char: Option<u8> = None;

    // Translation map
    let mut map = [0u8; 256];
    for i in 0..256 {
        map[i] = i as u8;
    }

    let mut delete_set = [false; 256];
    let mut squeeze_set = [false; 256];

    if args.delete {
        for &b in &s1 {
            delete_set[b as usize] = true;
        }
        if let Some(ref s2_expanded) = args.set2 {
            // Squeeze set from SET2 if delete and squeeze are both set
            for &b in &expand_set(s2_expanded) {
                squeeze_set[b as usize] = true;
            }
        }
    } else {
        // Translation
        let mut s1_final = s1.clone();
        if args.truncate && s1.len() > s2.len() {
            s1_final.truncate(s2.len());
        }

        for (i, &b) in s1_final.iter().enumerate() {
            let target = if i < s2.len() {
                s2[i]
            } else {
                *s2.last().unwrap_or(&b)
            };
            map[b as usize] = target;
        }

        if args.squeeze {
            let squeeze_source = if args.set2.is_some() { &s2 } else { &s1 };
            for &b in squeeze_source {
                squeeze_set[b as usize] = true;
            }
        }
    }

    loop {
        let n = match stdin_lock.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("tr: {}", e);
                return 1;
            }
        };

        for &b in &buf[..n] {
            if args.delete && delete_set[b as usize] {
                continue;
            }

            let out_char = map[b as usize];

            if args.squeeze && squeeze_set[out_char as usize] {
                if Some(out_char) == last_char {
                    continue;
                }
            }

            if let Err(e) = stdout_lock.write_all(&[out_char]) {
                eprintln!("tr: {}", e);
                return 1;
            }
            last_char = Some(out_char);
        }
    }

    0
}

fn expand_set(s: &str) -> Vec<u8> {
    let mut res = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => res.push(b'\n'),
                't' => res.push(b'\t'),
                'r' => res.push(b'\r'),
                'v' => res.push(b'\x0b'),
                'f' => res.push(b'\x0c'),
                'b' => res.push(b'\x08'),
                '\\' => res.push(b'\\'),
                '0'..='7' => {
                    // Octal escape
                    let mut val = 0u8;
                    let mut count = 0;
                    while i + 1 < chars.len() && count < 3 {
                        if let Some(digit) = chars[i + 1].to_digit(8) {
                            val = val * 8 + digit as u8;
                            i += 1;
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    res.push(val);
                    i -= 1; // adjust for outer i+=1
                }
                _ => res.push(chars[i + 1] as u8),
            }
            i += 2;
        } else if i + 2 < chars.len() && chars[i + 1] == '-' {
            // Range
            let start = chars[i] as u8;
            let end = chars[i + 2] as u8;
            for b in start..=end {
                res.push(b);
            }
            i += 3;
        } else {
            res.push(chars[i] as u8);
            i += 1;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_set() {
        assert_eq!(expand_set("abc"), b"abc");
        assert_eq!(expand_set("a-c"), b"abc");
        assert_eq!(expand_set("\\n\\t"), b"\n\t");
        assert_eq!(expand_set("\\012"), b"\n");
    }
}
