use std::collections::HashMap;

use crate::{request::Request, response::Response, XpressError};
use derivative::Derivative;

pub(crate) type Handler =
    Box<dyn Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync>;

#[derive(Derivative)]
#[derivative(Debug)]
struct TrieNode {
    route_segment: Segment,
    #[derivative(Debug = "ignore")]
    handler: Option<Handler>,
    is_leaf: bool,
    children: Box<NodeChildren>,
}

impl TrieNode {
    fn new(route_segment: Segment) -> Self {
        TrieNode {
            route_segment,
            handler: None,
            is_leaf: false,
            children: NodeChildren::new(),
        }
    }
}

#[derive(Debug)]
struct NodeChildren {
    static_nodes: HashMap<String, TrieNode>,
    dynamic_node: Option<TrieNode>,
}

impl NodeChildren {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            static_nodes: HashMap::new(),
            dynamic_node: None,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Router {
    routes: Vec<TrieNode>,
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

        let root = if let Some(n) = self.routes.iter_mut().find(
            |node| matches!(&node.route_segment, Segment::Static(m) if m == &route_def.method),
        ) {
            n
        } else {
            self.routes
                .push(TrieNode::new(Segment::Static(route_def.method.clone())));
            self.routes.last_mut().unwrap()
        };

        let mut cur = root;
        for seg in route_def.segments {
            // Skip empty segments (handles root path "/")
            if let Segment::Static(ref path) = seg {
                if path.is_empty() {
                    continue;
                }
            }

            match &seg {
                Segment::Static(path) => {
                    if !cur.children.static_nodes.contains_key(path) {
                        cur.children
                            .static_nodes
                            .insert(path.clone(), TrieNode::new(seg.clone()));
                    }
                    cur = cur.children.static_nodes.get_mut(path).unwrap();
                }
                Segment::Dynamic(_) => {
                    if cur.children.dynamic_node.is_none() {
                        cur.children.dynamic_node = Some(TrieNode::new(seg.clone()));
                    }
                    cur = cur.children.dynamic_node.as_mut().unwrap();
                }
            }
        }

        cur.handler = Some(handler);
        cur.is_leaf = true;
        Ok(())
    }

    pub(crate) fn resolve(
        &self,
        method: String,
        path: String,
    ) -> Option<(&Handler, HashMap<String, String>)> {
        // Find method root
        let root = self
            .routes
            .iter()
            .find(|node| matches!(&node.route_segment, Segment::Static(m) if m == &method))?;

        let segments: Vec<&str> = path[1..].split('/').filter(|s| !s.is_empty()).collect();

        let mut cur = root;
        let mut params = HashMap::new();

        for seg in segments {
            // Try static first, then dynamic
            cur = cur.children.static_nodes.get(seg).or_else(|| {
                cur.children.dynamic_node.as_ref().and_then(|node| {
                    if let Segment::Dynamic(param_name) = &node.route_segment {
                        params.insert(param_name.clone(), seg.to_string());
                        Some(node)
                    } else {
                        None
                    }
                })
            })?;
        }

        // Return handler only if this is a leaf node
        if cur.is_leaf {
            cur.handler.as_ref().map(|h| (h, params))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Segment {
    Static(String),
    Dynamic(String),
}

pub(crate) struct RouteDef {
    // path: String,
    method: String,
    segments: Vec<Segment>,
}

impl RouteDef {
    fn new() -> Self {
        RouteDef {
            // path: String::new(),
            method: String::new(),
            segments: Vec::new(),
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

        route_def.segments = crate::parser::parse_path_segments(path_part);

        Ok(route_def)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{request::Request, response::Response};

    // Helper function to create a simple handler
    fn dummy_handler() -> Handler {
        Box::new(|_req: &Request, _res: &mut Response| Ok(()))
    }

    // Helper function to create a handler that can be identified
    fn handler_with_id(id: &'static str) -> Handler {
        Box::new(move |_req: &Request, res: &mut Response| {
            res.send(id.to_string())?;
            Ok(())
        })
    }

    #[test]
    fn test_register_static_route() {
        let mut router = Router::new();
        let result = router.register_route("GET /users".to_string(), dummy_handler());
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_dynamic_route() {
        let mut router = Router::new();
        let result = router.register_route("GET /users/:id".to_string(), dummy_handler());
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_nested_route() {
        let mut router = Router::new();
        let result = router.register_route(
            "GET /api/users/:id/posts/:post_id".to_string(),
            dummy_handler(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_multiple_methods() {
        let mut router = Router::new();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("POST /users".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("DELETE /users".to_string(), dummy_handler())
            .unwrap();

        assert_eq!(router.routes.len(), 3);
    }

    #[test]
    fn test_resolve_static_route() {
        let mut router = Router::new();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();

        let result = router.resolve("GET".to_string(), "/users".to_string());
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_resolve_dynamic_route() {
        let mut router = Router::new();
        router
            .register_route("GET /users/:id".to_string(), dummy_handler())
            .unwrap();

        let result = router.resolve("GET".to_string(), "/users/123".to_string());
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_resolve_multiple_dynamic_params() {
        let mut router = Router::new();
        router
            .register_route(
                "GET /users/:user_id/posts/:post_id".to_string(),
                dummy_handler(),
            )
            .unwrap();

        let result = router.resolve("GET".to_string(), "/users/42/posts/99".to_string());
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("user_id"), Some(&"42".to_string()));
        assert_eq!(params.get("post_id"), Some(&"99".to_string()));
    }

    #[test]
    fn test_static_route_priority_over_dynamic() {
        let mut router = Router::new();
        router
            .register_route("GET /users/:id".to_string(), handler_with_id("dynamic"))
            .unwrap();
        router
            .register_route("GET /users/new".to_string(), handler_with_id("static"))
            .unwrap();

        // Static route should match first
        let result = router.resolve("GET".to_string(), "/users/new".to_string());
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert!(params.is_empty()); // No dynamic params captured

        // Dynamic route should still work for other values
        let result = router.resolve("GET".to_string(), "/users/123".to_string());
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_resolve_nonexistent_route() {
        let mut router = Router::new();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();

        let result = router.resolve("GET".to_string(), "/posts".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_wrong_method() {
        let mut router = Router::new();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();

        let result = router.resolve("POST".to_string(), "/users".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_incomplete_path() {
        let mut router = Router::new();
        router
            .register_route("GET /users/:id/posts".to_string(), dummy_handler())
            .unwrap();

        // Path is incomplete - stops at /users/123 instead of going to /posts
        let result = router.resolve("GET".to_string(), "/users/123".to_string());
        assert!(result.is_none()); // Should be None because it's not a leaf
    }

    #[test]
    fn test_resolve_extra_path_segments() {
        let mut router = Router::new();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();

        // Extra segments that don't exist in route
        let result = router.resolve("GET".to_string(), "/users/extra".to_string());
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_routes_same_prefix() {
        let mut router = Router::new();
        router
            .register_route("GET /api/users".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("GET /api/posts".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("GET /api/comments".to_string(), dummy_handler())
            .unwrap();

        assert!(router
            .resolve("GET".to_string(), "/api/users".to_string())
            .is_some());
        assert!(router
            .resolve("GET".to_string(), "/api/posts".to_string())
            .is_some());
        assert!(router
            .resolve("GET".to_string(), "/api/comments".to_string())
            .is_some());
    }

    #[test]
    fn test_root_route() {
        let mut router = Router::new();
        router
            .register_route("GET /".to_string(), dummy_handler())
            .unwrap();

        let result = router.resolve("GET".to_string(), "/".to_string());
        assert!(result.is_some());
    }

    #[test]
    fn test_empty_path_segments() {
        let mut router = Router::new();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();

        // Path with trailing slash creates empty segment
        let segments: Vec<&str> = "/users/"[1..].split('/').collect();
        assert_eq!(segments, vec!["users", ""]);
    }

    #[test]
    fn test_complex_routing_tree() {
        let mut router = Router::new();

        // Register multiple routes forming a complex tree
        router
            .register_route("GET /".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("GET /users".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("GET /users/:id".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("GET /users/:id/profile".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("POST /users".to_string(), dummy_handler())
            .unwrap();
        router
            .register_route("DELETE /users/:id".to_string(), dummy_handler())
            .unwrap();

        // Test all routes resolve correctly
        assert!(router.resolve("GET".to_string(), "/".to_string()).is_some());
        assert!(router
            .resolve("GET".to_string(), "/users".to_string())
            .is_some());
        assert!(router
            .resolve("GET".to_string(), "/users/123".to_string())
            .is_some());
        assert!(router
            .resolve("GET".to_string(), "/users/123/profile".to_string())
            .is_some());
        assert!(router
            .resolve("POST".to_string(), "/users".to_string())
            .is_some());
        assert!(router
            .resolve("DELETE".to_string(), "/users/456".to_string())
            .is_some());
    }

    #[test]
    fn test_dynamic_param_names_preserved() {
        let mut router = Router::new();
        router
            .register_route(
                "GET /articles/:article_id/comments/:comment_id".to_string(),
                dummy_handler(),
            )
            .unwrap();

        let result = router.resolve(
            "GET".to_string(),
            "/articles/my-article/comments/my-comment".to_string(),
        );
        assert!(result.is_some());

        let (_, params) = result.unwrap();
        assert_eq!(params.get("article_id"), Some(&"my-article".to_string()));
        assert_eq!(params.get("comment_id"), Some(&"my-comment".to_string()));
    }
}
