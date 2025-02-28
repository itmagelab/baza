#!/bin/bash

cd ${1:-$HOME/.password-store}

for i in $(git ls-tree -r master --name-only | grep -vE '^\.(git|gpg)'); do
  source=${i%%.gpg}
  target=${source//\//::}
  echo "=> rename ${source}"
  echo "   to ${target}"
  pass ${source} | baza --stdin "${target}"
done
