#!/usr/bin/env bcc-lua

local ffi = require('ffi')
local bpf = require('bpf')
local S = require('syscall')

local map = bpf.map('array', 1)

local probe = bpf.kprobe('myprobe:sys_write', function (ptregs)
    map[0] = ptregs.ax
end)

pcall(function()
    probe:attach()
    S.sleep(1)
end)
