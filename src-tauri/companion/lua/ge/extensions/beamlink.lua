-- BeamLink companion extension (runs inside BeamNG.drive, next to BeamMP).
--
-- The BeamMP launcher has no way to be told "join this server" from the
-- outside, so BeamLink leaves a request in settings/beamlink/autojoin.json
-- and this extension hands it to BeamMP's own MPCoreNetwork.connectToServer
-- once BeamMP is connected to its launcher. Everything else (auth, mod
-- downloads, the security prompt for server mods) stays BeamMP's job.
--
-- It also writes settings/beamlink/status.json every two seconds so the
-- BeamLink window can show "in game / on server X".

local M = {}

local DIR = "/settings/beamlink"
local JOIN_FILE = DIR .. "/autojoin.json"
local STATUS_FILE = DIR .. "/status.json"
-- A request older than this is from a launch that never got this far.
local MAX_AGE = 600
-- BeamMP logs in right after its launcher connects; give that a moment so
-- the join is not refused as "not logged in".
local SETTLE = 3

local timer = 0
local statusTimer = 0
local connectedFor = 0

local function mp()
  return rawget(_G, "MPCoreNetwork")
end

local function tryJoin()
  if not FS:fileExists(JOIN_FILE) then return end
  local net = mp()
  if not net or not net.isLauncherConnected or not net.isLauncherConnected() then
    connectedFor = 0
    return
  end
  if connectedFor < SETTLE then return end

  local job = jsonReadFile(JOIN_FILE)
  FS:removeFile(JOIN_FILE)
  if type(job) ~= "table" or not job.ip or not job.port then
    log("W", "beamlink", "ignoring malformed join request")
    return
  end
  if job.created and (os.time() - job.created) > MAX_AGE then
    log("W", "beamlink", "ignoring stale join request for " .. tostring(job.ip))
    return
  end
  log("I", "beamlink", "joining " .. tostring(job.name) .. " (" .. job.ip .. ":" .. tostring(job.port) .. ")")
  guihooks.trigger("toastrMsg", {type = "info", title = "BeamLink", msg = "Joining " .. tostring(job.name or job.ip)})
  net.connectToServer(job.ip, tonumber(job.port), job.name or job.ip, false)
end

local function writeStatus()
  local net = mp()
  local server = net and net.getCurrentServer and net.getCurrentServer() or nil
  local status = {
    time = os.time(),
    beammp = net ~= nil,
    launcher = net ~= nil and net.isLauncherConnected and net.isLauncherConnected() or false,
    session = net ~= nil and net.isMPSession and net.isMPSession() or false,
    version = beamng_versionb,
  }
  if type(server) == "table" then
    status.server = { ip = server.ip, port = server.port, name = server.name }
  end
  jsonWriteFile(STATUS_FILE, status, false)
end

local function onUpdate(dt)
  timer = timer + dt
  statusTimer = statusTimer + dt
  if timer >= 1 then
    local net = mp()
    if net and net.isLauncherConnected and net.isLauncherConnected() then
      connectedFor = connectedFor + timer
    end
    timer = 0
    local ok, err = pcall(tryJoin)
    if not ok then log("E", "beamlink", "join failed: " .. tostring(err)) end
  end
  if statusTimer >= 2 then
    statusTimer = 0
    pcall(writeStatus)
  end
end

local function onExtensionLoaded()
  if not FS:directoryExists(DIR) then FS:directoryCreate(DIR) end
  log("I", "beamlink", "BeamLink companion loaded")
end

M.onUpdate = onUpdate
M.onExtensionLoaded = onExtensionLoaded

return M
