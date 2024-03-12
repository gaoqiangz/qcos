//! 接口客户端，所有的操作都基于该对象
use crate::acl::AclHeader;
use crate::request::Response;
use crate::signer::Signer;
/// 接口请求Client
/// # Examples
/// ```
/// use qcos::client::Client;
/// let client = Client::new("secrect_id", "secrect_key", "bucket", "region");
/// assert_eq!(client.get_host(), "bucket.cos.region.myqcloud.com");
///```
use chrono::Utc;
use std::collections::HashMap;

pub struct Client {
    secrect_id: String,
    secrect_key: String,
    bucket: String,
    region: String,
    pub(crate) speed_limiter: async_speed_limit::Limiter,
}

impl Client {
    pub fn new(
        secrect_id: impl Into<String>,
        secrect_key: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
    ) -> Self {
        Self {
            secrect_id: secrect_id.into(),
            secrect_key: secrect_key.into(),
            bucket: bucket.into(),
            region: region.into(),
            speed_limiter: async_speed_limit::Limiter::new(f64::INFINITY),
        }
    }

    pub fn with_limiter(
        secrect_id: impl Into<String>,
        secrect_key: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
        speed_limiter: async_speed_limit::Limiter,
    ) -> Self {
        Self {
            secrect_id: secrect_id.into(),
            secrect_key: secrect_key.into(),
            bucket: bucket.into(),
            region: region.into(),
            speed_limiter,
        }
    }

    pub fn get_host(&self) -> String {
        format!("{}.cos.{}.myqcloud.com", self.bucket, self.region)
    }

    pub fn get_secrect_key(&self) -> &str {
        &self.secrect_key
    }
    pub fn get_secrect_id(&self) -> &str {
        &self.secrect_id
    }

    // 生成通用的request headers, 包含`Host`及`Date`
    pub fn gen_common_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), self.get_host().to_string());
        let now_str = Utc::now().format("%a, %d %b %Y %T GMT").to_string();
        headers.insert("Date".to_string(), now_str);
        headers
    }

    pub fn get_full_url_from_path(&self, path: &str) -> String {
        format!("https://{}{}", self.get_host(), path)
    }

    pub fn get_path_from_object_key(&self, key: &str) -> String {
        let mut url_path = key.to_string();
        if !url_path.starts_with("/") {
            url_path = format!("/{}", url_path);
        }
        url_path
    }
    // 生成查询bucket list的host
    pub fn get_host_for_bucket_query(&self) -> String {
        if self.region.is_empty() {
            return "service.cos.myqcloud.com".to_string();
        }
        format!("cos.{}.myqcloud.com", self.region)
    }

    // 返回带有`Authorization` 的headers, 如果headers从参数传入, 除添加acl头部之外不会添加其他头
    // 否则以`gen_common_headers` 返回作为初始值
    pub fn get_headers_with_auth(
        &self,
        method: &str,
        url_path: &str,
        acl_header: Option<&AclHeader>,
        orgin_headers: Option<HashMap<String, String>>,
        query: Option<&HashMap<String, String>>,
    ) -> HashMap<String, String> {
        let mut headers;
        if let Some(origin_headers) = orgin_headers {
            headers = origin_headers;
        } else {
            headers = self.gen_common_headers();
        }
        if let Some(acl_header) = acl_header {
            for (k, v) in acl_header.get_headers() {
                headers.insert(k.to_string(), v.to_string());
            }
        }
        let signature = Signer::new(method, url_path, Some(&headers), query).get_signature(
            self.get_secrect_key(),
            self.get_secrect_id(),
            7200,
        );
        headers.insert("Authorization".to_string(), signature);
        headers
    }

    pub fn make_response(&self, resp: Result<Response, Response>) -> Response {
        match resp {
            Ok(e) => e,
            Err(e) => e,
        }
    }
}
