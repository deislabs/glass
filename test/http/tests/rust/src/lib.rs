use deislabs_http_v01::{Request, Response};

witx_bindgen_rust::export!("../../deislabs_http_v01.witx");

struct DeislabsHttpV01 {}

impl deislabs_http_v01::DeislabsHttpV01 for DeislabsHttpV01 {
    fn handler(req: Request) -> Response {
        let (method, uri, headers, _, _) = req;
        println!(
            "method: {:?}\nuri: {:?}\nheaders: {:?}",
            method, uri, headers
        );

        (
            200,
            None,
            Some("Nerva, Trajan, Hadrian, Pius, and Marcus Aurelius are the five best emperors. Don't @ me.".as_bytes().to_vec())
        )
    }
}
