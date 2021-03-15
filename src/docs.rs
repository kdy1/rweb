use crate::openapi::Spec;
use warp::filters::BoxedFilter;
use warp::Filter;
use warp::Reply;

/// Helper filter that exposes an openapi spec on the `/docs` endpoint.
///
/// # Example - single use with data injection
///
/// ```ignore
/// let (spec, filter) = openapi::spec().build(|| index());
/// serve(filter.or(openapi_docs(spec)))
/// .run(([127, 0, 0, 1], 3030))
/// .await;
/// ```
pub fn openapi_docs(spec: Spec) -> BoxedFilter<(impl Reply,)> {
    let docs_openapi = warp::path("openapi.json").map(move || warp::reply::json(&spec.to_owned()));
    let docs = warp::path("docs").map(|| {
        return warp::reply::html(
            r#"
            <!doctype html>
            <html lang="en">
            <head>
            <title>rweb</title>
            <link href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@3/swagger-ui.css" rel="stylesheet">
            </head>
            <body>
                <div id="swagger-ui"></div>
                <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@3/swagger-ui-bundle.js" charset="UTF-8"> </script>
                <script>
                    window.onload = function() {
                    const ui = SwaggerUIBundle({
                        "dom_id": "\#swagger-ui",
                        presets: [
                        SwaggerUIBundle.presets.apis,
                        SwaggerUIBundle.SwaggerUIStandalonePreset
                        ],
                        layout: "BaseLayout",
                        deepLinking: true,
                        showExtensions: true,
                        showCommonExtensions: true,
                        url: "/openapi.json",
                    })
                    window.ui = ui;
                };
            </script>
            </body>
            </html>
        "#,
        );
    });
    docs.or(docs_openapi).boxed()
}
