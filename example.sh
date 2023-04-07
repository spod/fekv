#!/bin/bash

echo '$ curl --fail -X "GET" localhost:3000/kv/foo'
curl --fail -X "GET" localhost:3000/kv/foo
echo '$ curl --fail -X "PUT" localhost:3000/kv/foo -d "bar"'
curl --fail -X "PUT" localhost:3000/kv/foo -d "bar"
echo
echo '$ curl --fail -X "GET" localhost:3000/kv/foo'
curl --fail -X "GET" localhost:3000/kv/foo
echo
echo '$ curl --fail -X "DELETE" localhost:3000/kv/foo'
curl --fail -X "DELETE" localhost:3000/kv/foo
echo
echo '$ curl --fail -X "POST" localhost:3000/kv/foo -d "BAR"'
curl --fail -X "POST" localhost:3000/kv/foo -d "BAR"
echo
echo '$ curl --fail -X "GET" localhost:3000/kv/foo'
curl --fail -X "GET" localhost:3000/kv/foo
echo
echo '$ curl --fail -X "POST" localhost:3000/kv/foo -d "baz"'
curl --fail -X "POST" localhost:3000/kv/foo -d "baz"
echo
echo '$ curl --fail -X "GET" localhost:3000/kv/foo'
curl --fail -X "GET" localhost:3000/kv/foo
echo
echo '$ curl --fail -X "DELETE" localhost:3000/kv/foo'
curl --fail -X "DELETE" localhost:3000/kv/foo
echo
echo '$ curl --fail -X "GET" localhost:3000/kv/foo'
curl --fail -X "GET" localhost:3000/kv/foo
