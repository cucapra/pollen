import tomllib
import requests
import os

GRAPHS_TOML = os.path.join(os.path.dirname(__file__), "graphs.toml")
SIZE_NAMES = {
    0: "",
    3: "k",
    6: "M",
    9: "G",
    12: "T",
}


def fmt_size(count):
    for scale, name in reversed(SIZE_NAMES.items()):
        unit = 10**scale
        if count > unit:
            return "{:.0f}{}B".format(count / unit, name)


def show_sizes():
    with open(GRAPHS_TOML, "rb") as f:
        graphs_data = tomllib.load(f)

    for category, graphs in graphs_data.items():
        for name, url in graphs.items():
            res = requests.head(url)
            length = int(res.headers["Content-Length"])
            print(category, name, fmt_size(length))


if __name__ == "__main__":
    show_sizes()
