#!/bin/bash

echo '$ curl --fail -X "GET" localhost:3000/fekv/foo'
curl --fail -X "GET" localhost:3000/fekv/foo
echo '$ curl --fail -X "PUT" localhost:3000/fekv/foo -d "bar"'
curl --fail -X "PUT" localhost:3000/fekv/foo -d "bar"
echo
echo '$ curl --fail -X "GET" localhost:3000/fekv/foo'
curl --fail -X "GET" localhost:3000/fekv/foo
echo
echo '$ curl --fail -X "DELETE" localhost:3000/fekv/foo'
curl --fail -X "DELETE" localhost:3000/fekv/foo
echo
echo '$ curl --fail -X "POST" localhost:3000/fekv/foo -d "BAR"'
curl --fail -X "POST" localhost:3000/fekv/foo -d "BAR"
echo
echo '$ curl --fail -X "GET" localhost:3000/fekv/foo'
curl --fail -X "GET" localhost:3000/fekv/foo
echo
echo '$ curl --fail -X "POST" localhost:3000/fekv/foo -d "baz"'
curl --fail -X "POST" localhost:3000/fekv/foo -d "baz"
echo
echo '$ curl --fail -X "GET" localhost:3000/fekv/foo'
curl --fail -X "GET" localhost:3000/fekv/foo
echo
echo '$ curl --fail -X "DELETE" localhost:3000/fekv/foo'
curl --fail -X "DELETE" localhost:3000/fekv/foo
echo
echo '$ curl --fail -X "GET" localhost:3000/fekv/foo'
curl --fail -X "GET" localhost:3000/fekv/foo
