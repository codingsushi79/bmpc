-- BeamLink companion extension (runs inside BeamNG.drive, next to BeamMP).
--
-- BeamLink leaves a join request in settings/beamlink/autojoin.json; this
-- extension carries it out the way a player would in BeamMP's own menu:
--
--   1. wait for BeamMP to connect to its launcher
--   2. open BeamMP's screen, so its Terms / sign-in steps run if needed and
--      its listeners (loading overlay, mod-download security prompt) exist
--   3. wait until the player is signed in or chose "Play as guest"
--   4. switch to BeamMP's server screen and join
--
-- Joining from the plain main menu skipped step 2: BeamMP's prompts had no
-- screen to appear on, so a join that needed the player's OK (e.g. to
-- download a server's mods) waited forever, half-connected.
--
-- Progress, and any failure or kick message BeamMP reports, is written to
-- settings/beamlink/status.json for the BeamLink window to show.

local M = {}

local DIR = "/settings/beamlink"
local JOIN_FILE = DIR .. "/autojoin.json"
local STATUS_FILE = DIR .. "/status.json"
-- A request older than this is from a launch that never got this far.
local MAX_AGE = 600
-- How long to wait for sign-in before giving up on a request.
local LOGIN_TIMEOUT = 600
-- Joining includes mod downloads, which can be slow; give up after this.
local JOIN_TIMEOUT = 900
-- Messages from BeamMP's loading flow that mean the join is over.
local FAILURE_WORDS = { "fail", "kick", "denied", "invalid", "refused", "timed out", "timeout", "banned", "full", "error", "closed" }

local job = nil          -- { ip, port, name, created }
local stage = "idle"     -- wait_launcher | wait_login | connect | joining | done | failed
local stageAt = 0
local message = nil      -- last progress/failure text for BeamLink
local lastLoading = nil  -- latest text from BeamMP's loading flow
local lastLoadingAt = 0
local hinted = false
local timer = 0
local statusTimer = 0

local function now() return os.time() end

local function mp()
  return rawget(_G, "MPCoreNetwork")
end

local function setStage(next, text)
  stage = next
  stageAt = now()
  message = text
end

local function toast(kind, text)
  guihooks.trigger("toastrMsg", { type = kind, title = "BeamLink", msg = text })
end

local function navigate(route)
  local router = extensions and extensions.ui_router
  if router and router.navigate then
    pcall(router.navigate, route)
  else
    guihooks.trigger("ChangeState", { state = route })
  end
end

local function launcherUp(net)
  return net and net.isLauncherConnected and net.isLauncherConnected() or false
end

-- Signed in or guest: BeamMP marks both by a successful auth result.
-- (getAuthResult has no side effects; isLoggedIn fires a UI event per call.)
local function signedIn(net)
  local auth = net and net.getAuthResult and net.getAuthResult()
  return type(auth) == "table" and (auth.success == true or auth.Auth == 1)
end

local function inSession(net)
  return net and net.isMPSession and net.isMPSession() or false
end

-- See what BeamMP's join flow reports (download progress, kick reasons):
-- it passes everything through UI.updateLoading.
local function hookLoading()
  local ui = rawget(_G, "UI")
  if not ui or type(ui.updateLoading) ~= "function" or ui.__beamlinkWrapped then return end
  local original = ui.updateLoading
  ui.updateLoading = function(data)
    if type(data) == "string" and data:sub(1, 1) == "l" then
      lastLoading = data:sub(2)
      lastLoadingAt = now()
    end
    return original(data)
  end
  ui.__beamlinkWrapped = true
end

local function looksLikeFailure(text)
  if type(text) ~= "string" then return false end
  local lower = text:lower()
  if lower:find("downloading resource", 1, true) then return false end
  for _, word in ipairs(FAILURE_WORDS) do
    if lower:find(word, 1, true) then return true end
  end
  return false
end

local function readRequest()
  if not FS:fileExists(JOIN_FILE) then return end
  local request = jsonReadFile(JOIN_FILE)
  FS:removeFile(JOIN_FILE)
  if type(request) ~= "table" or not request.ip or not request.port then
    log("W", "beamlink", "ignoring malformed join request")
    return
  end
  if request.created and (now() - request.created) > MAX_AGE then
    log("W", "beamlink", "ignoring stale join request for " .. tostring(request.ip))
    return
  end
  job = request
  hinted = false
  lastLoading = nil
  setStage("wait_launcher", "Waiting for BeamMP to connect to its launcher")
  log("I", "beamlink", "join requested: " .. tostring(job.name) .. " (" .. job.ip .. ":" .. tostring(job.port) .. ")")
end

local function step()
  readRequest()
  if not job then return end
  local net = mp()
  local elapsed = now() - stageAt

  if stage == "wait_launcher" then
    if launcherUp(net) then
      -- BeamMP's own screen runs its launcher check, Terms and sign-in
      -- steps as needed, and mounts the prompts a join may raise.
      navigate("menu.beammp")
      setStage("wait_login", "Waiting for you to sign in or choose Play as guest in BeamMP")
    elseif elapsed > MAX_AGE then
      setStage("failed", "BeamMP never connected to its launcher")
      job = nil
    end

  elseif stage == "wait_login" then
    if signedIn(net) then
      navigate("menu.beammp.servers")
      setStage("connect", "Joining " .. tostring(job.name or job.ip))
    elseif elapsed > 4 and not hinted then
      hinted = true
      toast("info", "Sign in or choose Play as guest in BeamMP to join " .. tostring(job.name or job.ip))
    elseif elapsed > LOGIN_TIMEOUT then
      setStage("failed", "Gave up: nobody signed in to BeamMP")
      job = nil
    end

  elseif stage == "connect" then
    -- Give the server screen a moment to mount before the join starts.
    if elapsed >= 1 then
      toast("info", "Joining " .. tostring(job.name or job.ip))
      lastLoading = nil
      net.connectToServer(job.ip, tonumber(job.port), job.name or job.ip, false)
      setStage("joining", "Connecting to " .. tostring(job.name or job.ip))
    end

  elseif stage == "joining" then
    if inSession(net) then
      setStage("done", "On " .. tostring(job.name or job.ip))
      job = nil
    elseif lastLoading and lastLoadingAt >= stageAt and looksLikeFailure(lastLoading) then
      setStage("failed", lastLoading)
      toast("error", "Couldn't join: " .. lastLoading)
      job = nil
    elseif elapsed > JOIN_TIMEOUT then
      setStage("failed", "Joining took too long" .. (lastLoading and (": " .. lastLoading) or ""))
      job = nil
    elseif lastLoading and lastLoadingAt >= stageAt then
      message = lastLoading
    end
  end
end

local function writeStatus()
  local net = mp()
  local server = net and net.getCurrentServer and net.getCurrentServer() or nil
  local status = {
    time = now(),
    beammp = net ~= nil,
    launcher = launcherUp(net),
    session = inSession(net),
    signed_in = signedIn(net),
    version = beamng_versionb,
    join = { stage = stage, message = message, name = job and job.name or nil, at = stageAt },
  }
  if type(server) == "table" then
    status.server = { ip = server.ip, port = server.port, name = server.name }
  end
  jsonWriteFile(STATUS_FILE, status, false)
end

local function onUpdate(dt)
  timer = timer + dt
  statusTimer = statusTimer + dt
  if timer >= 0.5 then
    timer = 0
    hookLoading()
    local ok, err = pcall(step)
    if not ok then
      log("E", "beamlink", "join step failed: " .. tostring(err))
      setStage("failed", "BeamLink companion error: " .. tostring(err))
      job = nil
    end
  end
  if statusTimer >= 1 then
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
