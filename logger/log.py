#!/usr/bin/env python
# -*- coding: utf-8 -*- 

"""
Logger for Aladin Lite
"""


import os, sys
import cgitb
import cgi
from time import strftime
import fcntl
import traceback

scriptDir = os.path.split(os.path.realpath(sys.argv[0]))[0]

#### Retrieving parameters ####
form = cgi.FieldStorage()
if not form.has_key('action'):
    print 'Content-Type: application/javascript\r\n'
    sys.exit()
else:
    action = form.getfirst('action')
callback = form.getfirst('callback', None)

params = form.getfirst('params', '')
pageUrl = form.getfirst('pageUrl', '')
referer = form.getfirst('referer', '') 

logfile = 'aladin-lite-log.txt'

isodate = strftime("%Y-%m-%dT%H:%M:%S")
try:
    if os.environ.has_key('REMOTE_ADDR'):
        ip = os.environ['REMOTE_ADDR']
    else:
        ip = 'Unknown'
    if os.environ.has_key('HTTP_USER_AGENT'):
        userAgent = os.environ['HTTP_USER_AGENT']
    else:
        userAgent = 'Unknown'
    logline = isodate+'\t'+ip+'\t'+action+'\t'+params+'\t'+userAgent+'\t'+referer+'\t'+pageUrl

    h = open(logfile, 'a')
    fcntl.lockf(h, fcntl.LOCK_EX)
    h.write(logline+'\n')
    fcntl.lockf(h, fcntl.LOCK_UN)
    h.close()
except:
    traceback.print_exc()
    # Do nothing
    #pass

if callback:
    print 'Content-Type: application/javascript\r\n'
    print '%s({});' % (callback)
    sys.exit()
else:
    print 'Content-Type: application/javascript\r\n'
    sys.exit()

