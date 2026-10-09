# Render every screen of the app without a display, with GPUI's Metal test renderer,
# and save each as a PNG. States that follow a click are reached by real clicks,
# keys and typing in that renderer. Run from app/:
#   python3 scripts/snapshots.py          build the snap variant, then every shot
#   python3 scripts/snapshots.py 10 13    only shots whose name starts with these
import os, subprocess, sys
subprocess.run(['cargo', 'build', '--features', 'snap', '--target-dir', 'target-snap'], check=True)
BIN = 'target-snap/debug/doneright-app'
P = {'DR_PAUSED': '1', 'DR_QUIET': '1'}
SHOTS = [
    ('01-welcome', {}, ''),
    ('02-setup', {'DR_START': 'setup', 'DR_SETUP_T': '3.0', 'DR_PAUSED': '1'}, ''),
    ('03-flow-checks', {**P, 'DR_START': 'flow', 'DR_T': '12.2'}, ''),
    ('04-flow-sent-back', {**P, 'DR_START': 'flow', 'DR_T': '17.5'}, ''),
    ('05-flow-taste-call', {**P, 'DR_START': 'flow', 'DR_T': '29'}, ''),
    ('06-flow-playbook', {**P, 'DR_START': 'flow', 'DR_FLOW': '1', 'DR_T': '8'}, ''),
    ('07-flow-extension', {**P, 'DR_START': 'flow', 'DR_FLOW': '2', 'DR_T': '8'}, ''),
    ('08-suggestion-arrives', {'DR_START': 'flow', 'DR_T': '12.2', 'DR_PAUSED': '1', 'DR_TOAST': 'propose'}, ''),
    ('09-flow-suggested', {**P, 'DR_START': 'flow', 'DR_FLOW': '3', 'DR_T': '8'}, ''),
    ('10-show-added', {**P, 'DR_START': 'flow', 'DR_T': '17.5', 'DR_DTAB': '1'}, 'click:show-added'),
    ('11-where-it-came-from', {**P, 'DR_START': 'flow', 'DR_T': '17.5'}, 'click:show-added; click:mark#0'),
    ('12-suggestion-added', {**P, 'DR_START': 'flow', 'DR_T': '29', 'DR_HOLD': '1'}, 'click:p-add'),
    ('13-palette', {**P, 'DR_START': 'flow', 'DR_T': '29'}, 'press:cmd-k; input:A gadget that shows open PRs per repo'),
    ('14-added-panel', {**P, 'DR_START': 'ext:0', 'DR_T': '29'}, 'click:show-added'),
    ('15-installed', {**P, 'DR_START': 'add', 'DR_BUILD': '1', 'DR_T': '29', 'DR_HOLD': '1'}, 'click:install#0'),
    ('16-needs-you', {**P, 'DR_START': 'needs', 'DR_T': '29'}, ''),
    ('17-sessions', {**P, 'DR_START': 'sessions', 'DR_T': '29'}, ''),
    ('18-map', {**P, 'DR_START': 'map', 'DR_T': '29'}, ''),
    ('19-add-anything', {**P, 'DR_START': 'add', 'DR_BUILD': '1', 'DR_T': '29'}, ''),
    ('20-learned', {**P, 'DR_START': 'learned', 'DR_T': '29'}, ''),
    ('21-wiki', {**P, 'DR_START': 'wiki', 'DR_T': '29'}, ''),
    ('22-numbers', {**P, 'DR_START': 'numbers', 'DR_T': '29'}, ''),
]
only = sys.argv[1:]
out_dir = os.environ.get('OUT', 'snapshots')
os.makedirs(out_dir, exist_ok=True)
for name, env, script in SHOTS:
    if only and not any(name.startswith(o) for o in only):
        continue
    path = os.path.join(out_dir, f'{name}.png')
    e = {k: v for k, v in os.environ.items() if not k.startswith('DR_')}
    e.update(env, DR_SNAP=path, DR_DO=script)
    r = subprocess.run([BIN], env=e, capture_output=True, text=True, timeout=180)
    notes = [l for l in (r.stdout + r.stderr).splitlines() if l.startswith('snap:') and 'snapshots/' not in l or 'panicked' in l]
    print(name, 'ok' if r.returncode == 0 and os.path.exists(path) else f'FAILED ({r.returncode})', *notes, sep='\n  ' if notes else ' ')
