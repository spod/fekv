#!/usr/bin/env python3

import requests

words = open("/usr/share/dict/words").readlines()
words = [word.strip() for word in words]

for word in words:
    requests.post(f"http://127.0.0.1:3000/fekv/{word}", data=word)