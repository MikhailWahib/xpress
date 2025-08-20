use crate::router::Segment;
use std::collections::HashMap;

pub(crate) fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

pub(crate) fn parse_path_segments(path: &str) -> Vec<Segment> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.starts_with(':') {
                Segment::Dynamic(s[1..].to_string())
            } else {
                Segment::Static(s.to_string())
            }
        })
        .collect()
}
