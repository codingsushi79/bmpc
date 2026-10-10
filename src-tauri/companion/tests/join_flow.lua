-- Exercises the companion's join flow against stand-ins for BeamNG and BeamMP.
-- Run from src-tauri/companion:  luajit tests/join_flow.lua
-- Minimal stand-ins for BeamNG/BeamMP globals the companion touches.
local files, written = {}, nil
FS = {
  fileExists = function(_, p) return files[p] ~= nil end,
  removeFile = function(_, p) files[p] = nil end,
  directoryExists = function() return true end,
  directoryCreate = function() end,
}
jsonReadFile = function(p) return files[p] end
jsonWriteFile = function(p, t) written = t end
log = function() end
toasts, routes = {}, {}
guihooks = { trigger = function(name, data) if name == "toastrMsg" then toasts[#toasts+1] = data.msg end end }
extensions = { ui_router = { navigate = function(r) routes[#routes+1] = r end } }
beamng_versionb = "0.39"
local launcher, auth, session, joined = false, {}, false, nil
MPCoreNetwork = {
  isLauncherConnected = function() return launcher end,
  getAuthResult = function() return auth end,
  isMPSession = function() return session end,
  getCurrentServer = function() return nil end,
  connectToServer = function(ip, port, name, skip) joined = {ip, port, name, skip} end,
}
UI = { updateLoading = function() end }

local clock = 1000
os.time = function() return clock end
local M = dofile("lua/ge/extensions/beamlink.lua")
local function tick(seconds) for _ = 1, seconds * 2 do clock = clock + 0.5; M.onUpdate(0.5) end end
local function check(cond, what) print((cond and "ok   " or "FAIL ") .. what); if not cond then os.exit(1) end end

files["/settings/beamlink/autojoin.json"] = { ip = "1.2.3.4", port = 30814, name = "Test", created = clock }
tick(2)
check(written.join.stage == "wait_launcher", "waits for the launcher first")
launcher = true; tick(1)
check(routes[1] == "menu.beammp", "opens BeamMP's screen once the launcher is up")
check(written.join.stage == "wait_login", "then waits for sign-in")
tick(6)
check(#toasts == 1 and toasts[1]:find("Play as guest"), "hints to sign in or pick guest")
check(joined == nil, "does not join before sign-in")
auth = { success = true }; tick(3)
check(routes[2] == "menu.beammp.servers", "moves to the server screen")
check(joined and joined[1] == "1.2.3.4" and joined[2] == 30814 and joined[4] == false, "joins with BeamMP's mod prompt kept on")
UI.updateLoading("lDownloading Resource 1/2: a.zip (40.0%)"); tick(1)
check(written.join.stage == "joining", "downloads are progress, not failure")
UI.updateLoading("lYou have been kicked: Invalid key"); tick(1)
check(written.join.stage == "failed" and written.join.message:find("kicked"), "a kick is reported as the failure reason")

-- Second run: straight into a session.
files["/settings/beamlink/autojoin.json"] = { ip = "5.6.7.8", port = 1, name = "Two", created = clock }
joined = nil; tick(2); tick(2)
session = true; tick(1)
check(written.join.stage == "done", "a session counts as success")
check(written.session == true, "status reports the session")
print("all good")
