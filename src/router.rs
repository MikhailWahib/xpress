use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use crate::{request::Request, response::Response, XpressError};

pub(crate) type Handler =
    Box<dyn Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync>;

struct Route {
    _path: String,
    method: String,
    segments: Vec<Segment>,
    handler: Handler, // method -> handler
}

#[derive(Debug)]
enum Segment {
    Static(String),
    Dynamic(String),
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("segments", &self.segments)
            .field("handler", &"<handler>")
            .finish()
    }
}

impl Route {
    pub(crate) fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let req_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if req_segments.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (route_seg, req_seg) in self.segments.iter().zip(req_segments) {
            match route_seg {
                Segment::Static(s) if s == req_seg => {}
                Segment::Static(_) => return None,
                Segment::Dynamic(name) => {
                    params.insert(name.clone(), req_seg.to_string());
                }
            }
        }

        Some(params)
    }
}

pub(crate) struct RouteDef {
    path: String,
    method: String,
    segments: Vec<Segment>,
    query_params: HashMap<String, String>,
}

impl RouteDef {
    fn new() -> Self {
        RouteDef {
            path: String::new(),
            method: String::new(),
            segments: Vec::new(),
            query_params: HashMap::new(),
        }
    }
}

impl TryFrom<&str> for RouteDef {
    type Error = XpressError;

    // Example -> "GET /path/:slug"
    fn try_from(route_str: &str) -> Result<Self, XpressError> {
        let mut route_def = RouteDef::new();

        let mut route_iter = route_str.split(" ");
        let (Some(method), Some(route)) = (route_iter.next(), route_iter.next()) else {
            return Err(Self::Error::ParsingError(format!(
                "Error parsing route {:?}",
                route_iter
            )));
        };

        route_def.method = method.to_string();

        let mut route = route.split("?");
        let Some(path_part) = route.next() else {
            return Err(Self::Error::ParsingError(format!(
                "Error parsing route {:?}",
                route_iter
            )));
        };

        path_part
            .split('/')
            .filter(|p| !p.is_empty())
            .for_each(|p| {
                if p.starts_with(':') {
                    route_def
                        .segments
                        .push(Segment::Dynamic(p[1..].to_string()));
                } else {
                    route_def.segments.push(Segment::Static(p.to_string()));
                }
            });

        if let Some(query_part) = route.next() {
            route_def.query_params = query_part
                .split('&')
                .filter_map(|pair| {
                    let mut kv = pair.split('=');
                    Some((
                        kv.next()?.to_string(),
                        kv.next().unwrap_or_default().to_string(),
                    ))
                })
                .collect();
        }

        Ok(route_def)
    }
}

impl RouteDef {
    fn with_handler(self, handler: Handler) -> Route {
        Route {
            _path: self.path,
            method: self.method,
            segments: self.segments,
            handler,
        }
    }
}

pub(crate) struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub(crate) fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub(crate) fn register_route(
        &mut self,
        route_str: String,
        handler: Handler,
    ) -> Result<(), XpressError> {
        let route_def = RouteDef::try_from(route_str.as_str())?;

        let route = route_def.with_handler(handler);
        self.routes.push(route);
        Ok(())
    }

    pub fn resolve<'a>(
        &self,
        method: String,
        path: String,
    ) -> Option<(&Handler, HashMap<String, String>)> {
        for route in &self.routes {
            if route.method == method {
                if let Some(params) = route.matches(&path) {
                    return Some((&route.handler, params));
                }
            }
        }
        None
    }
}
