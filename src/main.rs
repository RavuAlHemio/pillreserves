mod model;
mod util;


use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{Infallible, TryInto};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Read};
use std::net::SocketAddr;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use env_logger;
use form_urlencoded;
use http::header::IF_MODIFIED_SINCE;
use hyper::{Body, Method, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use log::error;
use num_rational::Rational64;
use num_traits::Zero;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use serde_json;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use toml;
use url::Url;

use crate::model::{Config, Drug, DrugToDisplay};
use crate::util::{BrFilter, FracToFloat, FracToStr, parse_decimal};


const HTTP_TIMESTAMP_FORMAT: &'static str = "%a, %d %b %Y %H:%M:%S GMT";


static CONFIG: OnceCell<RwLock<Config>> = OnceCell::new();
static IMAGE_PATH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(
    "^/images/(?P<filename>[A-Za-z0-9-_]+[.][A-Za-z0-9]+)$"
).expect("failed to compile regex"));


async fn load_data() -> Option<Vec<Drug>> {
    let data_path = {
        let config_guard = CONFIG
            .get().expect("config is not set")
            .read().await;
        config_guard.data_path.clone()
    };
    let reader = match File::open(&data_path) {
        Ok(r) => r,
        Err(e) => {
            error!("failed to open file: {}", e);
            return None;
        },
    };

    match serde_json::from_reader(reader) {
        Ok(vd) => Some(vd),
        Err(e) => {
            error!("failed to load data: {}", e);
            None
        },
    }
}

async fn store_data(data: &[Drug]) -> bool {
    let data_path = {
        let config_guard = CONFIG
            .get().expect("config is not set")
            .read().await;
        config_guard.data_path.clone()
    };
    let writer = match File::create(&data_path) {
        Ok(r) => r,
        Err(e) => {
            error!("failed to open file: {}", e);
            return false;
        },
    };

    match serde_json::to_writer_pretty(writer, data) {
        Ok(()) => true,
        Err(e) => {
            error!("failed to store data: {}", e);
            false
        },
    }
}

fn respond_500() -> Result<Response<Body>, Infallible> {
    let resp_body = Body::from("500 Something Went Wrong On The Server");
    let resp = Response::builder()
        .status(500)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(resp_body)
        .expect("failed to build body");
    Ok(resp)
}

fn respond_304() -> Result<Response<Body>, Infallible> {
    let resp_res = Response::builder()
        .status(304)
        .body(Body::empty());
    match resp_res {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("failed to assemble 304 response body: {}", e);
            return respond_500();
        },
    }
}

fn respond_400(message: &str) -> Result<Response<Body>, Infallible> {
    let resp_body = Body::from(format!("400 Bad Request: {}", message));
    let resp_res = Response::builder()
        .status(400)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(resp_body);
    match resp_res {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("failed to assemble 400 response body: {}", e);
            return respond_500();
        },
    }
}

fn respond_403() -> Result<Response<Body>, Infallible> {
    let resp_body = Body::from("403 Forbidden; token missing or invalid");
    let resp_res = Response::builder()
        .status(403)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(resp_body);
    match resp_res {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("failed to assemble 403 response body: {}", e);
            return respond_500();
        },
    }
}

fn respond_404() -> Result<Response<Body>, Infallible> {
    let resp_body = Body::from("404 Not Found; where the h*ck is it?");
    let resp_res = Response::builder()
        .status(404)
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(resp_body);
    match resp_res {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("failed to assemble 404 response body: {}", e);
            return respond_500();
        },
    }
}

fn respond_405(allowed: &str) -> Result<Response<Body>, Infallible> {
    let resp_body = Body::from(format!("405 Wrong Method; try one of: {}", allowed));
    let resp_res = Response::builder()
        .status(405)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Allowed", allowed)
        .body(resp_body);
    match resp_res {
        Ok(resp) => Ok(resp),
        Err(e) => {
            error!("failed to assemble 405 response body: {}", e);
            return respond_500();
        },
    }
}

async fn handle_get(_request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let template = r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
<meta charset="utf-8" />
<title>Pill Reserves</title>
<style type="text/css">
body { font-family: sans-serif; }
table, th, td { border: 1px solid #ccc; }
th, td { padding: 0.2em 0.4em; vertical-align: top; }
td.count { text-align: right; }
td.components ul { margin-top: 0; margin-bottom: 0; padding-inline-start: 15px; }
form.replenish input[name=amount] { width: 3em; }
@media (color) {
    th { background-color: #603; color: #fff; }
}
@media print {
    th.replenish, td.replenish { display: none; }
    form { display: none; }
}
</style>
</head>
<body>
<h1>Pill Reserves</h1>
<table>
<tr>
    <th class="obverse-photo">Obverse</th>
    <th class="reverse-photo">Reverse</th>
    <th class="trade-name">Trade name</th>
    <th class="components">Components</th>
    <th class="description">Description</th>
    <th class="remaining">Remaining</th>
    <th class="dosage">Dosage</th>
    <th class="replenish">Replenish</th>
</tr>
{% for dtd in drugs_to_display %}
{% if dtd.drug.show %}
<tr>
    <td class="obverse-photo">
        {%- if dtd.drug.obverse_photo -%}
            <img src="images/{{ dtd.drug.obverse_photo|urlencode_strict|escape }}" width="100" height="80" />
        {%- endif -%}
    </td>
    <td class="reverse-photo">
        {%- if dtd.drug.reverse_photo -%}
            <img src="images/{{ dtd.drug.reverse_photo|urlencode_strict|escape }}" width="100" height="80" />
        {%- endif -%}
    </td>
    <td class="trade-name">{{ dtd.drug.trade_name|escape }}</td>
    <td class="components">
        <ul>
        {% for component in dtd.drug.components %}
            <li>
                <span class="generic-name">{{ component.generic_name|escape }}</span>
                <span class="amount">{{ component.amount|frac2float }}</span>
                <span class="unit">{{ component.unit|escape }}</span>
            </li>
        {% endfor %}
        </ul>
    </td>
    <td class="description">{{ dtd.drug.description|escape|br }}</td>
    <td class="remaining">
        <span class="total">{{ dtd.drug.remaining|frac2float }}</span>
        {% if dtd.remaining_weeks is number %}
            (<span class="weeks">{{ dtd.remaining_weeks }}</span>)
        {% endif %}
    </td>
    <td class="dosage">
        <span class="morning">{{ dtd.drug.dosage_morning|frac2str|escape }}</span>
        &#8210;
        <span class="noon">{{ dtd.drug.dosage_noon|frac2str|escape }}</span>
        &#8210;
        <span class="evening">{{ dtd.drug.dosage_evening|frac2str|escape }}</span>
        &#8210;
        <span class="night">{{ dtd.drug.dosage_night|frac2str|escape }}</span>
    </td>
    <td class="replenish">
        <form method="post" class="replenish">
            <input type="hidden" name="do" value="replenish" />
            <input type="hidden" name="drug-index" value="{{ dtd.index }}" />
            <input type="number" name="amount" step="0.01" />
            <input type="submit" value="Replenish" />
        </form>
    </td>
</tr>
{% endif %}
{% endfor %}
</table>

<p>
    <form method="post" class="take-week">
        <input type="hidden" name="do" value="take-week" />
        <input type="submit" value="Reduce by a week" />
    </form>
</p>
</body>
</html>
"#;

    let data = match load_data().await {
        None => return respond_500(),
        Some(d) => d,
    };

    let data_to_show: Vec<DrugToDisplay> = data.iter()
        .enumerate()
        .map(|(i, d)| {
            // add up total dosage per day
            let total_dosage_week = d.total_dosage_day() * Rational64::new(7, 1);
            let full_weeks = if *total_dosage_week.numer() > 0 {
                let doses_available = d.remaining() / total_dosage_week;
                Some(doses_available.numer() / doses_available.denom())
            } else {
                None
            };

            DrugToDisplay::new(i, d.clone(), full_weeks)
        })
        .filter(|dtd| dtd.drug().show())
        .collect();

    let mut tera: Tera = Default::default();
    tera.autoescape_on(vec![]);
    tera.register_filter("br", BrFilter);
    tera.register_filter("frac2str", FracToStr);
    tera.register_filter("frac2float", FracToFloat);
    let mut ctx = Context::new();
    ctx.insert("drugs_to_display", &data_to_show);
    let body_str = match tera.render_str(template, &ctx) {
        Ok(bs) => bs,
        Err(e) => {
            error!("error rendering template: {:?}", e);
            return respond_500();
        },
    };

    let resp_body = Body::from(body_str);
    let resp_res = Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(resp_body);
    match resp_res {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("failed to assemble response body: {}", e);
            return respond_500();
        },
    }
}

async fn handle_post(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (head, body) = request.into_parts();
    let body_bytes = match hyper::body::to_bytes(body).await {
        Ok(bb) => bb,
        Err(e) => {
            error!("failed to read request body: {}", e);
            return respond_500();
        },
    };
    let body_vec = body_bytes.to_vec();

    let opts: HashMap<String, String> = form_urlencoded::parse(&body_vec)
        .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()))
        .collect();

    let do_val = match opts.get("do") {
        Some(dv) => dv,
        None => return respond_400("missing value for \"do\""),
    };

    let mut data = match load_data().await {
        None => return respond_500(),
        Some(d) => d,
    };

    match do_val.as_str() {
        "replenish" => {
            let index_str = match opts.get("drug-index") {
                Some(s) => s,
                None => return respond_400("missing value for \"drug-index\""),
            };
            let index: usize = match index_str.parse() {
                Ok(i) => i,
                Err(_) => return respond_400("invalid value for \"drug-index\""),
            };
            if index >= data.len() {
                return respond_400("value for \"drug-index\" out of range");
            }

            let amount_str = match opts.get("amount") {
                Some(s) => s,
                None => return respond_400("missing value for \"amount\""),
            };
            let amount: Rational64 = match parse_decimal(amount_str) {
                Ok(i) => i,
                Err(_) => return respond_400("invalid value for \"amount\""),
            };
            match amount.cmp(&Zero::zero()) {
                Ordering::Less => {
                    let abs_amount = -amount;
                    data[index].reduce(&abs_amount);
                },
                Ordering::Equal => {
                    return respond_400("\"amount\" must not be 0");
                },
                Ordering::Greater => {
                    data[index].replenish(&amount);
                },
            }
        },
        "take-week" => {
            for drug in &mut data {
                let week_dose = drug.total_dosage_day() * Rational64::new(7, 1);
                drug.reduce(&week_dose);
            }
        },
        _other => {
            return respond_400("unknown value for \"do\"");
        },
    }

    // write updated data
    if !store_data(&data).await {
        return respond_500();
    }

    // redirect to myself
    let base_url_string = {
        let config_guard = CONFIG
            .get().expect("config is not set")
            .read().await;
        config_guard.base_url.clone()
    };
    let base_url: Url = match base_url_string.parse() {
        Ok(bu) => bu,
        Err(e) => {
            error!("failed to parse base URL {:?}: {}", base_url_string, e);
            return respond_500();
        },
    };

    let path_and_query = match head.uri.path_and_query() {
        Some(paq) => paq,
        None => {
            error!("failed to obtain path and query from request URL");
            return respond_500();
        },
    };
    let path_and_query_string = path_and_query.to_string();
    let relative_path_and_query = path_and_query_string.trim_start_matches('/');
    let my_url = match base_url.join(relative_path_and_query) {
        Ok(u) => u,
        Err(e) => {
            error!("failed to join path and query: {}", e);
            return respond_500();
        },
    };
    log::debug!("my_url: {}", my_url);

    let response_res = Response::builder()
        .status(302)
        .header("Location", my_url.to_string())
        .body(Body::from(""));
    match response_res {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("failed to assemble redirect response: {}", e);
            return respond_500();
        },
    }
}

async fn handle_get_image(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path_caps = match IMAGE_PATH_REGEX.captures(request.uri().path()) {
        Some(pc) => pc,
        None => return respond_404(),
    };
    let filename_match = path_caps.name("filename")
        .expect("unmatched filename capture");
    let filename = filename_match.as_str();

    let mut path = PathBuf::from("images");
    path.push(filename);

    let file_meta = match fs::metadata(&path) {
        Ok(fm) => fm,
        Err(e) => {
            return if e.kind() == io::ErrorKind::NotFound {
                respond_404()
            } else {
                error!("error obtaining file {:?} metadata: {}", filename, e);
                respond_500()
            };
        },
    };

    if let Some(ims) = request.headers().get(IF_MODIFIED_SINCE) {
        if let Ok(ims_str) = ims.to_str() {
            if let Ok(timestamp) = Utc.datetime_from_str(ims_str, HTTP_TIMESTAMP_FORMAT) {
                if let Ok(modified) = file_meta.modified() {
                    let modified_timestamp: DateTime<Utc> = modified.into();
                    if modified_timestamp <= timestamp {
                        return respond_304();
                    }
                }
            }
        }
    }

    let mut last_mod_text_opt: Option<String> = None;
    if let Ok(modified) = file_meta.modified() {
        let modified_timestamp: DateTime<Utc> = modified.into();
        last_mod_text_opt = Some(modified_timestamp.format(HTTP_TIMESTAMP_FORMAT).to_string());
    }

    // FIXME: stream the file?
    let file_bytes = {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return if e.kind() == io::ErrorKind::NotFound {
                    respond_404()
                } else {
                    error!("error opening file {:?}: {}", filename, e);
                    respond_500()
                };
            },
        };

        let mut buf = if let Ok(meta_len) = file_meta.len().try_into() {
            Vec::with_capacity(meta_len)
        } else {
            Vec::new()
        };
        if let Err(e) = file.read_to_end(&mut buf) {
            error!("error reading file {:?}: {}", filename, e);
            return respond_500();
        }

        buf
    };

    let content_type = if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    };

    let resp_len = file_bytes.len();
    let resp_body = Body::from(file_bytes);
    let mut resp_builder = Response::builder()
        .header("Content-Type", content_type)
        .header("Content-Length", resp_len.to_string());
    if let Some(lmt) = last_mod_text_opt {
        resp_builder = resp_builder.header("Last-Modified", lmt);
    }
    let resp_res = resp_builder
        .body(resp_body);
    match resp_res {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("failed to assemble response body: {}", e);
            return respond_500();
        },
    }
}

async fn handle_request(request: Request<Body>) -> Result<Response<Body>, Infallible> {
    let uri_path = request.uri().path();

    // unauthenticated endpoints first
    if uri_path.starts_with("/images/") {
        return if request.method() == Method::GET {
            handle_get_image(request).await
        } else {
            respond_405("GET")
        };
    }

    // authentication starts here

    // check for token
    let query_str = match request.uri().query() {
        None => return respond_403(),
        Some(q) => q,
    };
    let query_kv: HashMap<String, String> = form_urlencoded::parse(query_str.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let token_value = match query_kv.get("token") {
        None => return respond_403(),
        Some(tv) => tv,
    };

    let token_matches = {
        CONFIG
            .get().expect("config is not set")
            .read().await
            .auth_tokens
            .iter()
            .any(|t| t == token_value)
    };
    if !token_matches {
        return respond_403();
    }

    // authenticated-only endpoints beyond this line

    if request.method() == Method::GET {
        handle_get(request).await
    } else if request.method() == Method::POST {
        handle_post(request).await
    } else {
        respond_405("GET, POST")
    }
}


async fn perform() -> i32 {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() < 1 || args.len() > 2 {
        eprintln!("Usage: {:?} [CONFIGPATH.toml]", args[0]);
        return 1;
    }
    let config_path: PathBuf = if args.len() > 1 {
        args[1].clone().into()
    } else {
        "config.toml".into()
    };

    env_logger::init();

    // load config
    {
        let mut config_file = match File::open(&config_path) {
            Ok(f) => f,
            Err(e) => {
                error!("failed to open config file {:?}: {}", config_path, e);
                return 1;
            },
        };
        let mut config_string = String::new();
        if let Err(e) = config_file.read_to_string(&mut config_string) {
            error!("failed to read config file {:?}: {}", config_path, e);
            return 1;
        };
        let config: Config = match toml::from_str(&config_string) {
            Ok(c) => c,
            Err(e) => {
                error!("failed to parse config file {:?}: {}", config_path, e);
                return 1;
            },
        };
        if let Err(_) = CONFIG.set(RwLock::new(config)) {
            error!("failed to set initial config");
            return 1;
        }
    }

    let addr: SocketAddr = {
        let config_guard = CONFIG
            .get().expect("config is set")
            .read().await;

        match config_guard.listen_addr.parse() {
            Ok(a) => a,
            Err(e) => {
                error!("failed to parse listen address and port {:?}: {}", config_guard.listen_addr, e);
                return 1;
            },
        }
    };

    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });
    let server = Server::bind(&addr).serve(make_service);
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }

    0
}


#[tokio::main]
async fn main() {
    std::process::exit(perform().await)
}
