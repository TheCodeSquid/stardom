#!/usr/bin/env python

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


def write_rust_array(file, name, entries):
    file.write("pub const {0}: &[&str] = &[\n".format(name))
    for entry in entries:
        file.write("    \"{0}\",\n".format(entry))
    file.write("];\n\n")


def write_rust_kv(file, name, entries):
    file.write("pub const {0}: &[(&str, &str)] = &[\n".format(name))
    for k, v in entries:
        file.write("    (\"{0}\", \"{1}\"),\n".format(k, v))
    file.write("];\n\n")


with open("stardom-macros/src/web.rs", "w") as file:
    write_rust_array(file, "ELEMENTS", elements)
    write_rust_array(file, "ATTRS", attrs)
    write_rust_kv(file, "EVENTS", events)
