use chrono::{DateTime, TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;

use std::collections::HashMap;
use std::io;
use std::ops::Deref;

lazy_static! {
    static ref HELP_RE: Regex = Regex::new(r"^#\s+HELP\s+(\w+)\s+(.+)$").unwrap();
    static ref TYPE_RE: Regex = Regex::new(r"^#\s+TYPE\s+(\w+)\s+(\w+)").unwrap();
    static ref SAMPLE_RE: Regex = Regex::new(
        r"^(?P<name>\w+)(\{(?P<labels>[^}]+)\})?\s+(?P<value>\S+)(\s+(?P<timestamp>\S+))?"
    )
    .unwrap();
}

#[derive(Debug, Eq, PartialEq)]
pub enum LineInfo<'a> {
    Doc {
        metric_name: &'a str,
        doc: &'a str,
    },
    Type {
        metric_name: String,
        sample_type: SampleType,
    },
    Sample {
        metric_name: &'a str,
        labels: Option<&'a str>,
        value: &'a str,
        timestamp: Option<&'a str>,
    },
    Empty,
    Ignored,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SampleType {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Untyped,
}

impl SampleType {
    pub fn parse(s: &str) -> SampleType {
        match s {
            "counter" => SampleType::Counter,
            "gauge" => SampleType::Gauge,
            "histogram" => SampleType::Histogram,
            "summary" => SampleType::Summary,
            _ => SampleType::Untyped,
        }
    }
}

impl<'a> LineInfo<'a> {
    pub fn parse(line: &'a str) -> LineInfo<'a> {
        let line = line.trim();
        if line.len() == 0 {
            return LineInfo::Empty;
        }
        match HELP_RE.captures(line) {
            Some(ref caps) => {
                return match (caps.get(1), caps.get(2)) {
                    (Some(ref metric_name), Some(ref doc)) => LineInfo::Doc {
                        metric_name: metric_name.as_str(),
                        doc: doc.as_str(),
                    },
                    _ => LineInfo::Ignored,
                }
            }
            None => {}
        }
        match TYPE_RE.captures(line) {
            Some(ref caps) => {
                return match (caps.get(1), caps.get(2)) {
                    (Some(ref metric_name), Some(ref sample_type)) => {
                        let sample_type = SampleType::parse(sample_type.as_str());
                        LineInfo::Type {
                            metric_name: match sample_type {
                                SampleType::Histogram => format!("{}_bucket", metric_name.as_str()),
                                _ => metric_name.as_str().to_string(),
                            },
                            sample_type: sample_type,
                        }
                    }
                    _ => LineInfo::Ignored,
                }
            }
            None => {}
        }
        match SAMPLE_RE.captures(line) {
            Some(ref caps) => {
                return match (
                    caps.name("name"),
                    caps.name("labels"),
                    caps.name("value"),
                    caps.name("timestamp"),
                ) {
                    (Some(ref name), labels, Some(ref value), timestamp) => LineInfo::Sample {
                        metric_name: name.as_str(),
                        labels: labels.map_or(None, |c| Some(c.as_str())),
                        value: value.as_str(),
                        timestamp: timestamp.map_or(None, |c| Some(c.as_str())),
                    },
                    _ => LineInfo::Ignored,
                }
            }
            None => LineInfo::Ignored,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Sample {
    pub metric: String,
    pub value: Value,
    pub labels: Labels,
    pub timestamp: DateTime<Utc>,
}

fn parse_bucket(s: &str, label: &str) -> Option<f64> {
    if let Some(kv) = s.split(",").next() {
        let kvpair = kv.split("=").collect::<Vec<_>>();
        let (k, v) = (kvpair[0], kvpair[1].trim_matches('"'));
        if k == label {
            match parse_golang_float(v) {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[derive(Debug, PartialEq)]
pub struct HistogramCount {
    pub less_than: f64,
    pub count: f64,
}

#[derive(Debug, PartialEq)]
pub struct SummaryCount {
    pub quantile: f64,
    pub count: f64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Labels(HashMap<String, String>);

impl Labels {
    fn new() -> Labels {
        Labels(HashMap::new())
    }
    fn parse(s: &str) -> Labels {
        let mut l = HashMap::new();
        for kv in s.split(",") {
            let kvpair = kv.split("=").collect::<Vec<_>>();
            if kvpair.len() != 2 || kvpair[0].len() == 0 {
                continue;
            }
            l.insert(
                kvpair[0].to_string(),
                kvpair[1].trim_matches('"').to_string(),
            );
        }
        Labels(l)
    }
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(|ref x| x.as_str())
    }
}

impl Deref for Labels {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Counter(f64),
    Gauge(f64),
    Histogram(Vec<HistogramCount>),
    Summary(Vec<SummaryCount>),
    Untyped(f64),
}

impl Value {
    fn push_histogram(&mut self, h: HistogramCount) {
        match self {
            &mut Value::Histogram(ref mut hs) => hs.push(h),
            _ => {}
        }
    }
    fn push_summary(&mut self, s: SummaryCount) {
        match self {
            &mut Value::Summary(ref mut ss) => ss.push(s),
            _ => {}
        }
    }
}

#[derive(Debug, Default)]
pub struct Scrape {
    pub samples: Vec<Sample>,
    types: HashMap<String, SampleType>,
    buckets: HashMap<String, Sample>,
}

fn parse_golang_float(s: &str) -> Result<f64, <f64 as std::str::FromStr>::Err> {
    // calling to_lowercase is actually crazy slow in Rust, so we can just check
    // against these 3 values (case insensitive) and then parse anything else
    if unicase::eq_ascii(s, "nan") {
        Ok(f64::NAN)
    }
    else if unicase::eq_ascii(s, "inf") {
        Ok(f64::INFINITY)
    }
    else if unicase::eq_ascii(s, "-inf") {
        Ok(f64::NEG_INFINITY)
    }
    else {
        s.parse::<f64>()
    }
}

impl Scrape {
    pub fn parse<'a>(&mut self, lines: impl Iterator<Item = &'a str>) -> io::Result<()> {
        self.parse_at(lines, Utc::now())
    }
    pub fn parse_at<'a>(
        &mut self,
        lines: impl Iterator<Item = &'a str>,
        sample_time: DateTime<Utc>,
    ) -> io::Result<()> {
        self.samples.clear();
        self.types.clear();
        self.buckets.clear();

        for line in lines {
            match LineInfo::parse(&line) {
                LineInfo::Doc {
                    ref metric_name,
                    ref doc,
                } => {
                    // Ignore docs, I don't even get why they're part of prometheus metrics???
                }
                LineInfo::Type {
                    ref metric_name,
                    ref sample_type,
                } => {
                    self.types.insert(metric_name.to_string(), *sample_type);
                }
                LineInfo::Sample {
                    metric_name,
                    ref labels,
                    value,
                    timestamp,
                } => {
                    // Parse value or skip
                    let fvalue = match parse_golang_float(value) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!(
                                message="Failed to parse flat",
                                value=?value,
                                metric_name=?metric_name,
                            );
                            continue
                        }
                    };
                    // Parse timestamp or use given sample time
                    let timestamp = if let Some(Ok(ts_millis)) = timestamp.map(|x| x.parse::<i64>())
                    {
                        Utc.timestamp_millis(ts_millis)
                    } else {
                        sample_time
                    };
                    match (self.types.get(metric_name), labels) {
                        (Some(SampleType::Histogram), Some(labels)) => {
                            if let Some(lt) = parse_bucket(labels, "le") {
                                let sample =
                                    self.buckets.entry(metric_name.to_string()).or_insert(Sample {
                                        metric: metric_name.to_string(),
                                        labels: Labels::new(),
                                        value: Value::Histogram(vec![]),
                                        timestamp: timestamp,
                                    });
                                sample.value.push_histogram(HistogramCount {
                                    less_than: lt,
                                    count: fvalue,
                                })
                            }
                        }
                        (Some(SampleType::Summary), Some(labels)) => {
                            if let Some(q) = parse_bucket(labels, "quantile") {
                                let sample =
                                    self.buckets.entry(metric_name.to_string()).or_insert(Sample {
                                        metric: metric_name.to_string(),
                                        labels: Labels::new(),
                                        value: Value::Summary(vec![]),
                                        timestamp: timestamp,
                                    });
                                sample.value.push_summary(SummaryCount {
                                    quantile: q,
                                    count: fvalue,
                                })
                            }
                        }
                        (ty, labels) => self.samples.push(Sample {
                            metric: metric_name.to_string(),
                            labels: labels.map_or(Labels::new(), |x| Labels::parse(x)),
                            value: match ty {
                                Some(SampleType::Counter) => Value::Counter(fvalue),
                                Some(SampleType::Gauge) => Value::Gauge(fvalue),
                                _ => Value::Untyped(fvalue),
                            },
                            timestamp,
                        }),
                    };
                }
                _ => {}
            }
        }
        for (_k, v) in self.buckets.drain() {
            self.samples.push(v);
        }

        Ok(())
    }
}
