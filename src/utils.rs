use regex::Regex;

pub fn sqlite_date(s: String) -> Result<String, Box<dyn std::error::Error>> {
    let cleaned_date = s.replace("-", "");

    if cleaned_date.len() != 8 {
        return Err("Date must be in format MMDDYYYY or MM-DD-YYYY".into());
    }

    let month_str = &cleaned_date[..2];
    let day_str = &cleaned_date[2..4];
    let year_str = &cleaned_date[4..];

    let month = month_str.parse::<i32>()?;
    let day = day_str.parse::<i32>()?;
    let year = year_str.parse::<i32>()?;

    if month > 12 || month <= 0 {
        return Err("Month should be between 1 and 12".into());
    }
    if day > 31 || day <= 0 {
        return Err("Day should be between 1 and 31".into());
    }

    let date = format!("{}-{:02}-{:02}", year, month, day);
    Ok(date)
}

pub fn is_valid_phone(phone: &str) -> bool {
    /*
     Matches formats:
     123-456-7890
     (123) 456-7890
     123.456.7890
     123 456 7890
     (123)-456-7890
     1234567890
     +1 (123) 456-7890
     +1-123-456-7890
     1-123-456-7890
    */
    let phone_regex =
        Regex::new(r"^(?:\+?1[-. ]?)?\(?([0-9]{3})\)?[-. ]?([0-9]{3})[-. ]?([0-9]{4})$").unwrap();
    phone_regex.is_match(phone)
}

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_date(date: &str) -> bool {
    let d = date.replace("-", "");
    if d.len() != 8 {
        return false;
    }

    let date_regex = Regex::new(r"^(0[1-9]|1[0-2])(0[1-9]|[12][0-9]|3[01])[12][0-9]{3}$").unwrap();

    if !date_regex.is_match(d.as_str()) {
        return false;
    }

    let month: u32 = d[0..2].parse().unwrap();
    let day: u32 = d[2..4].parse().unwrap();
    let year: u32 = d[4..8].parse().unwrap();

    if [4, 6, 9, 11].contains(&month) && day > 30 {
        return false;
    }

    if month == 2 {
        let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        if (is_leap_year && day > 29) || (!is_leap_year && day > 28) {
            return false;
        }
    }

    true
}

pub fn from_sqlite_to_date(sqlite_date: String) -> String {
    let vs: Vec<&str> = sqlite_date.split("-").collect();
    let date: String = std::format!("{}-{}-{}", vs[1], vs[2], vs[0]);
    date
}
