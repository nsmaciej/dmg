#!/usr/bin/env bash
set -e
hdiutil create -ov -volname Test -nospotlight -noanyowners -srcfolder test -format UDRW Test
