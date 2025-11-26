use chrono::{DateTime, Timelike, Utc};

use crate::forecast_models::ForecastReport;
use crate::weather_models::Measurement;

//pub const TIME_LINE_FMT: &str = "%H:%M:%S%.3f %a"; // good for debugging
pub const TIME_LINE_FMT: &str = "  %H:%M:%S  %a ";
pub const DATE_LINE_FMT: &str = " %Y-%m-%d W%V ";
pub const TIME_SHORT_FMT: &str = "%H:%M";
pub const MAX_WEATHER_TIME_MINUTES: i64 = 120;
pub const MAX_STATION_CLOCK_MISALIGNMENT_MINUTES: i64 = 2;

fn fmt_opt<T, F>(val: &Option<T>, f: F) -> String
where
    F: FnOnce(&T) -> String,
{
    let Some(ref v) = val else {
        return "?".to_owned();
    };
    f(v)
}

pub fn format_min_temperature(report: &ForecastReport) -> String {
    let min_temperature_time = report.min_temperature_time.format(TIME_SHORT_FMT);
    format!(
        "{: >5.1}°C @ {} ",
        report.min_temperature, min_temperature_time
    )
}

pub fn format_max_temperature(report: &ForecastReport) -> String {
    let max_temperature_time = report.max_temperature_time.format(TIME_SHORT_FMT);
    format!(
        "{: >5.1}°C @ {} ",
        report.max_temperature, max_temperature_time
    )
}

pub fn format_precipitation(report: &ForecastReport) -> String {
    format!(
        "{:3.0}% {:2.0}h {: >5.1}mm",
        report.precipitation_max_probability, report.precipitation_sum, report.precipitation_hours
    )
}

pub fn format_wind(report: &ForecastReport) -> String {
    format!(
        "{:>3} {:>3.0}‥{:<3.0} km/h",
        report.wind_direction_at_max_speed, report.wind_max_speed, report.wind_max_gusts
    )
}

pub fn format_clouds(report: &ForecastReport) -> String {
    format!(
        "{:3.0}%⌀ {:3.0}%Δ {:2.0}UV",
        report.cloud_cover_min, report.cloud_cover_max, report.max_uv_index
    )
}

pub fn format_sunrise_sunset(report: &ForecastReport) -> String {
    let sunrise_hour = report.sunrise.hour();
    let sunrise_minute = report.sunrise.minute();
    let sunset = report.sunset.format(TIME_SHORT_FMT);

    if report.sunrise.hour() > 9 {
        format!("{:02.0}:{:02.0}‥{}", sunrise_hour, sunrise_minute, sunset)
    } else {
        let day_length_hours = report.day_length.num_hours();
        let day_length_minutes =
            report.day_length.num_minutes() - report.day_length.num_hours() * 60;
        format!(
            "{:1.0}:{:02.0}‥{} {:02}h{:02}",
            sunrise_hour, sunrise_minute, sunset, day_length_hours, day_length_minutes
        )
    }
}

pub fn format_weather_line0(measurement: &Measurement, now: &DateTime<Utc>) -> String {
    let minutes = if let Some(update_time) = measurement.update_time {
        let minutes = now
            .naive_utc()
            .signed_duration_since(update_time)
            .num_minutes();

        if minutes > MAX_WEATHER_TIME_MINUTES {
            "old ".to_owned()
        } else if minutes <= 0 {
            // Theoretically this shouldn't be possible, but
            // station clock may be out of sync with ours.
            if minutes < -MAX_STATION_CLOCK_MISALIGNMENT_MINUTES {
                "?'".to_owned()
            } else {
                "now ".to_owned()
            }
        } else {
            format!("{:3}'", minutes)
        }
    } else {
        "?'".to_owned()
    };

    let humidity = fmt_opt(&measurement.humidity, |h| format!("{:3.0}", h));
    let precipitation = fmt_opt(&measurement.precipitation, |p| format!("{:3.0}", p));
    format!("{:>4} {:>3}% {:>3} mm", minutes, humidity, precipitation,)
}

pub fn format_weather_line1(measurement: &Measurement) -> String {
    let wind_direction = fmt_opt(&measurement.wind_direction, |wd| wd.clone());
    let wind_speed = fmt_opt(&measurement.wind_speed, |ws| format!("{}", ws));
    let gusts_speed = fmt_opt(&measurement.gusts_speed, |gs| format!("{}", gs));

    format!(
        "{:>3} {:>3}‥{:<3} km/h",
        wind_direction, wind_speed, gusts_speed
    )
}
