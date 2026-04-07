use hyper::Request;

use crate::config::ServiceKind;

pub fn route<B>(req: &Request<B>) -> ServiceKind {
    if let Some(kind) = from_auth_header(req) {
        return kind;
    }
    if let Some(kind) = from_amz_target(req) {
        return kind;
    }
    ServiceKind::S3
}

fn from_auth_header<B>(req: &Request<B>) -> Option<ServiceKind> {
    let auth = req.headers().get("authorization")?.to_str().ok()?;
    let credential = auth.split("Credential=").nth(1)?;
    let service_name = credential.split('/').nth(3)?;
    ServiceKind::from_sigv4_name(service_name)
}

fn from_amz_target<B>(req: &Request<B>) -> Option<ServiceKind> {
    let target = req.headers().get("x-amz-target")?.to_str().ok()?;
    let prefix = target.split('.').next()?;
    if prefix.starts_with("DynamoDB") {
        ServiceKind::from_sigv4_name("dynamodb")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::Empty;
    use hyper::body::Bytes;

    fn empty_request() -> Request<Empty<Bytes>> {
        Request::builder().body(Empty::new()).unwrap()
    }

    fn request_with_auth(service: &str) -> Request<Empty<Bytes>> {
        Request::builder()
            .header(
                "authorization",
                format!(
                    "AWS4-HMAC-SHA256 Credential=AKID/20260408/us-east-1/{service}/aws4_request, \
                     SignedHeaders=host, Signature=abc"
                ),
            )
            .body(Empty::new())
            .unwrap()
    }

    fn request_with_amz_target(target: &str) -> Request<Empty<Bytes>> {
        Request::builder()
            .header("x-amz-target", target)
            .body(Empty::new())
            .unwrap()
    }

    #[test]
    fn sigv4_s3() {
        assert_eq!(route(&request_with_auth("s3")), ServiceKind::S3);
    }

    #[test]
    fn sigv4_unknown_defaults_to_s3() {
        assert_eq!(route(&request_with_auth("unknown")), ServiceKind::S3);
    }

    #[test]
    fn no_auth_defaults_to_s3() {
        assert_eq!(route(&empty_request()), ServiceKind::S3);
    }

    #[test]
    fn dynamodb_target() {
        let req = request_with_amz_target("DynamoDB_20120810.GetItem");
        assert_eq!(route(&req), ServiceKind::S3);
    }

    #[test]
    fn auth_takes_precedence_over_default() {
        assert_eq!(route(&request_with_auth("s3")), ServiceKind::S3);
    }

    #[test]
    fn presigned_url_defaults_to_s3() {
        let req = Request::builder()
            .uri("http://localhost:4566/bucket/key?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKID/20260408/us-east-1/s3/aws4_request")
            .body(Empty::<Bytes>::new())
            .unwrap();
        assert_eq!(route(&req), ServiceKind::S3);
    }
}
