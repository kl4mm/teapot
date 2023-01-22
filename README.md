# Teapot 🍵

This project is an attempt at making an E-commerce application using Hyper, which is a "low-level" http library for Rust. It leaves quite a few things up to the programmer, eg handling requests, routing, parsing query parameters etc. As a result it takes quite a bit more code to get an endpoint up and running compared to typical web frameworks.

## Query Library

Early on in the project I noticed how messy the code can become for GET endpoints that take required or optional query parameters. I found myself writing very specific code for each endpoint to validate parameters and generate the `WHERE` clause for my SQL. This motivated me to write a library which abstracts away the parsing and validation of query parameters. It also exposes a clean api to generate a `WHERE` clause with the parsed parameters.
