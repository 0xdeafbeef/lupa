#!/usr/bin/env lua
local ffi = require("ffi")
local M = { cache = {} }
local mt = {
  __index = function(_, key)
    return M.cache[key]
  end,
}
setmetatable(M.cache, mt)

local function memoize(name, loader)
  return function(key)
    local slot = name .. ":" .. key
    M.cache[slot] = M.cache[slot] or loader(key)
    return M.cache[slot]
  end
end

function M:configure(opts)
  self.loader = memoize("main", opts.loader or function(value) return value end)
end

pcall(function()
  M:configure({ loader = function(value) return tostring(value) end })
end)
