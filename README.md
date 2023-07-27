[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://img.shields.io/github/actions/workflow/status/esrlabs/someip/check.yml?branch=main)](https://github.com/esrlabs/someip/actions)
[![Docs](https://img.shields.io/badge/docs-here-green)](https://esrlabs.github.io/someip/someip_messages)

# SOME/IP

Scalable service-Oriented MiddlewarE over IP (SOME/IP)

SOME/IP is an automotive middleware solution that can be used for control messages.

## Parser

This project implements a pure rust parser for SOME/IP content.

## Features

The feature `url` enables conversion between [someip_messages::SdEndpointOption](https://esrlabs.github.io/someip/someip_messages/struct.SdEndpointOption.html) and [url::Url](https://docs.rs/url/2.2.0/url/struct.Url.html). The `url` feature is disabled by default.