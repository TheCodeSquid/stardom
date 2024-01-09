#!/usr/bin/env python

import json
import requests
from bs4 import BeautifulSoup

WHATWG_INDEX = "https://html.spec.whatwg.org/dev/indices.html"
WHATWG_MEDIA = "https://html.spec.whatwg.org/dev/media.html"
WHATWG_DND = "https://html.spec.whatwg.org/dev/dnd.html"

elements = []
attrs = []
events = []


def make_soup(url: str) -> BeautifulSoup:
    page = requests.get(url)
    return BeautifulSoup(page.text, "html5lib")


index = make_soup(WHATWG_INDEX)
media = make_soup(WHATWG_MEDIA)
dnd = make_soup(WHATWG_DND)

elem_table = index.find(id="elements-3").find_next_sibling("table")
for row in elem_table.tbody.find_all("tr"):
    if not row.th.code:
        continue
    name = row.th.code.text

    elements.append(name)


attr_table = index.find(id="attributes-3").find_next_sibling("table")
for row in attr_table.tbody.find_all("tr"):
    attr = row.th.code.text
    if attr not in attrs:
        attrs.append(attr)

event_table = index.find(id="events-2").find_next_sibling("table")
for row in event_table.tbody.find_all("tr"):
    name, interface, *other = row.find_all("td")
    events.append((name.code.text, interface.code.text))
event_tables = media.find(id="mediaevents").find_next_siblings("table", limit=6)
for table in event_tables:
    for row in table.tbody.find_all("tr"):
        name, interface, *other = row.find_all("td")
        name = name.code.text
        interface = interface.code.text
        
        if (name, interface) in events:
            continue
        events.append((name, interface))
event_table = dnd.find(id="dndevents").find_next_sibling("table")
for row in event_table.tbody.find_all("tr"):
    name = row.td.code.text
    events.append((name, "DragEvent"))

data = {
    "elements": elements,
    "attrs": attrs,
    "events": events
}
with open("stardom-macros/web.json", "w") as file:
    json.dump(data, file, indent=2)
