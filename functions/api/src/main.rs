use lambda_http::{
  service_fn, Body, Error as LambdaError, IntoResponse, Request, RequestExt,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use tracing_subscriber;

pub type Holidays = Vec<Holiday>;

#[derive(Deserialize, Clone)]
pub struct Holiday {
  pub name: String,
  #[serde(rename = "name_local")]
  pub name_local: String,
  pub language: String,
  pub description: String,
  pub country: String,
  pub location: String,
  #[serde(rename = "type")]
  pub type_field: String,
  pub date: String,
  #[serde(rename = "date_year")]
  pub date_year: String,
  #[serde(rename = "date_month")]
  pub date_month: String,
  #[serde(rename = "date_day")]
  pub date_day: String,
  #[serde(rename = "week_day")]
  pub week_day: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HolidayReduced {
  name: String,
  desc: String,
  date: String,
}

impl From<Holiday> for HolidayReduced {
  fn from(hol: Holiday) -> Self {
    Self {
      name: hol.name,
      desc: hol.description,
      date: hol.date,
    }
  }
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
  setup_logs();
  let func = service_fn(handler);
  lambda_http::run(func).await?;
  Ok(())
}

fn setup_logs() {
  match is_local() {
    true => tracing_subscriber::fmt::init(),
    false => tracing_subscriber::fmt().with_ansi(false).init(),
  }
}

fn is_local() -> bool {
  match std::env::var("ENVIRONMENT") {
    Ok(value) => value == "local",
    Err(_) => false,
  }
}

async fn handler(request: Request) -> Result<impl IntoResponse, LambdaError> {
  return match request.query_string_parameters().first("iso") {
    Some(country) => match request.query_string_parameters().first("year") {
      Some(year) => match request.query_string_parameters().first("month") {
        Some(month) => match request.query_string_parameters().first("day") {
          Some(day) => {
            return get_holiday(country, year, month, day).await;
          }
          None => Ok(
            json!({ "error": true, "message": "The query 'day' was not present." }),
          ),
        },
        None => Ok(
          json!({ "error": true, "message": "The query 'month' was not present." }),
        ),
      },
      None => Ok(
        json!({ "error": true, "message": "The query 'year' was not present." }),
      ),
    },
    None => {
      return Ok(
        json!({ "error": true, "message": "The query 'iso' was not present." }),
      );
    }
  };
  // match request.body() {
  //   Body::Text(value) => match serde_json::from_str::<UserData>(value) {
  //     Ok(data) => Ok(fn_data(&data)),
  //     Err(error) => {
  //       error!("ERROR: {}", error.to_string());
  //       Ok(json!({
  //         "error": true,
  //         "message": format!("Deserialization error: {}", error.to_string())
  //       }))
  //     }
  //   },
  //   _ => {
  //     error!("ERROR: Unknown error");
  //     Ok(json!({ "error": true, "message": "Unknown error" }))
  //   }
  // }
}

async fn get_holiday(
  country: &str,
  year: &str,
  month: &str,
  day: &str,
) -> Result<serde_json::Value, LambdaError> {
  match std::env::var("ABSTRACT_API") {
    Ok(key) => {
      let mut url = Url::parse("https://holidays.abstractapi.com/v1/")?;
      url.set_query(Some(&format!("api_key={}", key)));
      url.set_query(Some(&format!("country={}", country)));
      url.set_query(Some(&format!("year={}", year)));
      url.set_query(Some(&format!("month={}", month)));
      url.set_query(Some(&format!("day={}", day)));
      let t = reqwest::get(url).await?;
      let json = t.json::<Holidays>().await?;
      if json.is_empty() {
        return Ok(json!({ "error": true, "message": "No holidays" }));
      }
      let hol: HolidayReduced = json[0].clone().into();
      Ok(json!(hol))
    }
    Err(_) => Ok(
      json!({ "error": true, "message": "Key for AbstractAPI doesn't exist."}),
    ),
  }
}
