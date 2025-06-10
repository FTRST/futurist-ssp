![Responding with modified HTML](https://gitlab.com/terminallyill/rust-server-side-proxy-example/-/raw/master/example_images/example_url.png)

This project handles two pieces of the futurist framework, a server-side proxy used for [NetXplorer](https://github.com/ftrst/ftrst-netxplorer) and the quantum random number generator data stream for [Quantum Coin Flip](https://github.com/ftrst/ftrst-quantum-flip).

Included below is details for the HTML Proxy Server for NetXplorer. The QRNG stream is not documented, but the code is relatively simple to read through.

## HTML Proxy Server

This project acts as a middleware to extract HTML content from pages and re-serve them within a node environment.

It:
- Can reformat HTML for mobile and desktop pages
- Comes with a default CORS config
- Uses Rustls instead of OpenSSL
- Can be deployed for free using deta.space

### What is this for?

In creating a React project, I needed to implement something similar to an iFrame for an external website on a different domain.

However, external URLs within the iFrame could not be accessed, and, due to security limitations of iFrames, event tracking is not available if the server and origin are not owned by the same entity.

To solve this, I created a div within my React project that is filled with the HTML content of what was needing to be served within the iFrame. Because of CORS restrictions, this was not possible without implementing a middleware, or server-side proxy, to fetch the content and return the HTML.

### Challenges

In creating this, cases appeared where images were not referencing a direct URL (such as "_assets/myimage.png").

To solve this, multiple libraries were included to parse the base URL from the current page and use regex to find and replace any instances of image sources and external page anchors.

### Using the project
*Clone the repository*

```git clone https://github.com/ftrst/futurist-ssp```

*Pre-check the setup*

By default, this runs on port 8080. This can be customized within the main.rs under the port binding at the bottom.

If you are planning on hosting this, update the CORS policy within the main.rs with the URL that will be accessing it.

*Run the project*

In the same directory, run:

```cargo run```

This will download all the required packages, compile them, etc.

*Testing the endpoing*

Use cURL, Postman, etc to test. Once it's hosted it should expect something like:

```http://localhost:3000/fetch?url=https://example.com```

If the parameter is included, but does not include a value, static HTML can be sent from within the actix server. See below:

![No included url example](https://gitlab.com/terminallyill/rust-server-side-proxy-example/-/raw/master/example_images/example_default.png)

Replace the *http://localhost:3000* with your site if hosting externally. That's it!
