#!/bin/bash

bd list $* | grep '^mtg' | awk '{ print $1 }' | sort --version-sort | xargs -I {} bd show {}
