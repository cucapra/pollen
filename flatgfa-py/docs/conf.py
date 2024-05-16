# Make mygfa module available to autodoc.
import sys
import os

sys.path.insert(0, os.path.abspath(".."))

project = "flatgfa"
copyright = "2024, Capra Lab"
author = "Capra Lab"

extensions = ["sphinx.ext.autodoc"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

html_theme = "alabaster"

autodoc_member_order = "bysource"
autodoc_typehints_format = "short"
