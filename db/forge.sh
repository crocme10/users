#!/bin/bash

rm -fr ./init.sql
touch ./init.sql
for file in functions/*; do
  cat ${file} >> ./init.sql
done
