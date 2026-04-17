import React from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { Line2 } from 'three/examples/jsm/lines/Line2.js';
import { LineGeometry } from 'three/examples/jsm/lines/LineGeometry.js';
import { LineMaterial } from 'three/examples/jsm/lines/LineMaterial.js';
import URDFLoader from 'urdf-loader';

const URDF_PATH = '/resources/arm01/urdf/reBot-DevArm_description_fixend.urdf';
const DEFAULT_CAMERA_POS = new THREE.Vector3(1.15, 0.95, 1.2);
const DEFAULT_TARGET = new THREE.Vector3(0, 0.35, 0);
const TRAIL_MAX_POINTS = 1200;
const TRAIL_SAMPLE_MIN_DIST = 0.001;
const SAFE_ZERO_SEGMENT = [0, 0, 0, 0, 0, 0];

export function ArmUrdfViewer({
  jointTargets,
  resetViewSeq = 0,
  clearTrailSeq = 0,
  exportTrailSeq = 0,
  replaySeq = 0,
  replayStopSeq = 0,
  replayFinishSeq = 0,
  replaySpeed = 1,
  importedTrail = null,
  simMode = 'direct',
  trailColor = '#ff0000',
  trailStyle = 'mono',
  trailVisible = true,
  onReplayStateChange,
}) {
  const hostRef = React.useRef(null);
  const robotRef = React.useRef(null);
  const cameraRef = React.useRef(null);
  const controlsRef = React.useRef(null);
  const modeRef = React.useRef(simMode);
  const trailVisibleRef = React.useRef(trailVisible);
  const trailColorRef = React.useRef(trailColor);
  const trailStyleRef = React.useRef(trailStyle);
  const targetRef = React.useRef(jointTargets || {});
  const animJointRef = React.useRef({});
  const trailLineRef = React.useRef(null);
  const trailRainbowRef = React.useRef(null);
  const trailDotsRef = React.useRef(null);
  const trailTubeRef = React.useRef(null);
  const trailPointsRef = React.useRef([]);
  const trailFramesRef = React.useRef([]);
  const endEffectorRef = React.useRef(null);
  const trailHeadRef = React.useRef(null);
  const importedFramesRef = React.useRef([]);
  const replayRef = React.useRef({ active: false, frames: [], cursor: 0 });
  const replayBusyRef = React.useRef(false);
  const replaySpeedRef = React.useRef(Math.max(0.05, Number(replaySpeed) || 1));
  const tmpWorldRef = React.useRef(new THREE.Vector3());
  const [status, setStatus] = React.useState('loading');

  const setLinePositionsSafe = React.useCallback((line, flatPositions) => {
    if (!line?.geometry) return;
    if (!Array.isArray(flatPositions) || flatPositions.length < 6) {
      line.geometry.setPositions(SAFE_ZERO_SEGMENT);
      line.computeLineDistances?.();
      line.geometry.computeBoundingSphere?.();
      return;
    }
    line.geometry.setPositions(flatPositions);
    line.computeLineDistances?.();
    line.geometry.computeBoundingSphere?.();
  }, []);

  const applyTrajectoryColor = React.useCallback((hexColor) => {
    const color = new THREE.Color(hexColor || '#ff0000');
    const lineMat = trailLineRef.current?.material;
    const dotsMat = trailDotsRef.current?.material;
    const tubeMat = trailTubeRef.current?.material;
    if (lineMat?.color) lineMat.color.copy(color);
    if (dotsMat?.color) dotsMat.color.copy(color);
    if (tubeMat?.color) tubeMat.color.copy(color);
  }, []);

  const setReplayBusy = React.useCallback(
    (next) => {
      const busy = Boolean(next);
      if (replayBusyRef.current === busy) return;
      replayBusyRef.current = busy;
      if (typeof onReplayStateChange === 'function') onReplayStateChange(busy);
    },
    [onReplayStateChange],
  );

  const normalizeJointMap = React.useCallback((raw) => {
    if (!raw || typeof raw !== 'object') return null;
    const out = {};
    Object.entries(raw).forEach(([k, v]) => {
      const n = Number(v);
      if (!Number.isFinite(n)) return;
      const key = String(k || '');
      if (/^joint\d+$/i.test(key)) {
        out[key.toLowerCase()] = n;
        return;
      }
      if (/^\d+$/.test(key)) {
        out[`joint${Number(key)}`] = n;
      }
    });
    return Object.keys(out).length ? out : null;
  }, []);

  const captureJointSnapshot = React.useCallback(() => {
    const robot = robotRef.current;
    if (!robot?.joints) return null;
    const out = {};
    Object.entries(robot.joints).forEach(([name, j]) => {
      let v = Number.NaN;
      if (Array.isArray(j?.jointValue)) v = Number(j.jointValue[0]);
      else v = Number(j?.jointValue);
      if (!Number.isFinite(v)) v = Number(j?.angle);
      if (Number.isFinite(v)) out[name] = v;
    });
    return Object.keys(out).length ? out : null;
  }, []);

  const applyJointMap = React.useCallback((map) => {
    const robot = robotRef.current;
    if (!robot?.joints || !map || typeof map !== 'object') return;
    const nextAnim = { ...animJointRef.current };
    Object.entries(map).forEach(([jointName, targetRaw]) => {
      const target = Number(targetRaw);
      if (!Number.isFinite(target)) return;
      const j = robot.joints?.[jointName];
      if (!j?.setJointValue) return;
      j.setJointValue(target);
      nextAnim[jointName] = target;
    });
    animJointRef.current = nextAnim;
  }, []);

  const interpolateJointMap = React.useCallback((a, b, t) => {
    const aj = a && typeof a === 'object' ? a : null;
    const bj = b && typeof b === 'object' ? b : null;
    if (!aj && !bj) return null;
    const keys = new Set([...(aj ? Object.keys(aj) : []), ...(bj ? Object.keys(bj) : [])]);
    const out = {};
    keys.forEach((k) => {
      const av = Number(aj?.[k]);
      const bv = Number(bj?.[k]);
      if (Number.isFinite(av) && Number.isFinite(bv)) out[k] = av + (bv - av) * t;
      else if (Number.isFinite(av)) out[k] = av;
      else if (Number.isFinite(bv)) out[k] = bv;
    });
    return Object.keys(out).length ? out : null;
  }, []);

  const pushTrailPoint = React.useCallback((point, joints = null) => {
    const line = trailLineRef.current;
    const rainbowLine = trailRainbowRef.current;
    const dots = trailDotsRef.current;
    const tube = trailTubeRef.current;
    if (!line || !point) return;
    const rawPoints = trailPointsRef.current;
    const rawFrames = trailFramesRef.current;
    rawPoints.push(point.clone());
    rawFrames.push({
      pos: point.clone(),
      joints: normalizeJointMap(joints),
    });
    if (rawPoints.length > TRAIL_MAX_POINTS) rawPoints.splice(0, rawPoints.length - TRAIL_MAX_POINTS);
    if (rawFrames.length > TRAIL_MAX_POINTS) rawFrames.splice(0, rawFrames.length - TRAIL_MAX_POINTS);

    const curvePoints =
      rawPoints.length >= 3
        ? new THREE.CatmullRomCurve3(rawPoints, false, 'catmullrom', 0.18).getPoints(
            Math.min(3600, Math.max(24, (rawPoints.length - 1) * 4)),
          )
        : rawPoints;
    const flat = [];
    curvePoints.forEach((p) => {
      flat.push(p.x, p.y, p.z);
    });
    // LineGeometry requires at least 2 points (6 floats), otherwise it throws.
    if (flat.length < 6 && flat.length >= 3) {
      flat.push(flat[0], flat[1], flat[2]);
    }
    setLinePositionsSafe(line, flat);
    if (rainbowLine?.geometry) {
      const posArr = [];
      const colArr = [];
      const base = new THREE.Color(trailColorRef.current || '#ff0000');
      const hsl = { h: 0, s: 1, l: 0.5 };
      base.getHSL(hsl);
      curvePoints.forEach((p, i) => {
        posArr.push(p.x, p.y, p.z);
        let c = base;
        if (trailStyleRef.current === 'multi') {
          const t = curvePoints.length > 1 ? i / (curvePoints.length - 1) : 0;
          c = new THREE.Color().setHSL((hsl.h + 0.25 * t) % 1, Math.min(1, hsl.s * 0.95 + 0.05), 0.42 + 0.25 * t);
        }
        colArr.push(c.r, c.g, c.b);
      });
      rainbowLine.geometry.setAttribute('position', new THREE.Float32BufferAttribute(posArr, 3));
      rainbowLine.geometry.setAttribute('color', new THREE.Float32BufferAttribute(colArr, 3));
      rainbowLine.geometry.computeBoundingSphere();
    }
    if (dots?.geometry) {
      dots.geometry.setFromPoints(rawPoints);
      dots.geometry.computeBoundingSphere();
    }
    if (tube) {
      if (rawPoints.length >= 2) {
        const curve = new THREE.CatmullRomCurve3(rawPoints, false, 'catmullrom', 0.18);
        const tubularSegments = Math.min(1200, Math.max(24, rawPoints.length * 2));
        const nextGeom = new THREE.TubeGeometry(curve, tubularSegments, 0.0028, 10, false);
        const prevGeom = tube.geometry;
        tube.geometry = nextGeom;
        prevGeom?.dispose?.();
      } else {
        tube.geometry = new THREE.BufferGeometry();
      }
    }
  }, [setLinePositionsSafe, normalizeJointMap]);

  const clearTrail = React.useCallback(() => {
    trailPointsRef.current = [];
    trailFramesRef.current = [];
    if (trailLineRef.current?.geometry) {
      setLinePositionsSafe(trailLineRef.current, SAFE_ZERO_SEGMENT);
    }
    if (trailDotsRef.current?.geometry) {
      trailDotsRef.current.geometry.setFromPoints([]);
      trailDotsRef.current.geometry.computeBoundingSphere();
    }
    if (trailRainbowRef.current?.geometry) {
      trailRainbowRef.current.geometry.setAttribute('position', new THREE.Float32BufferAttribute(SAFE_ZERO_SEGMENT, 3));
      trailRainbowRef.current.geometry.setAttribute(
        'color',
        new THREE.Float32BufferAttribute([1, 0, 0, 1, 0, 0], 3),
      );
      trailRainbowRef.current.geometry.computeBoundingSphere();
    }
    if (trailTubeRef.current?.geometry) {
      trailTubeRef.current.geometry.dispose?.();
      trailTubeRef.current.geometry = new THREE.BufferGeometry();
    }
  }, [setLinePositionsSafe]);

  React.useEffect(() => {
    modeRef.current = simMode;
    const line = trailLineRef.current;
    const rainbowLine = trailRainbowRef.current;
    const dots = trailDotsRef.current;
    const tube = trailTubeRef.current;
    const canShow = simMode === 'trajectory' && trailVisibleRef.current;
    if (line) line.visible = canShow && trailStyleRef.current !== 'multi';
    if (rainbowLine) rainbowLine.visible = canShow;
    if (dots) dots.visible = canShow;
    if (tube) tube.visible = canShow;
    if (simMode === 'trajectory') {
      const robot = robotRef.current;
      if (robot?.joints) {
        const next = {};
        Object.entries(targetRef.current || {}).forEach(([jointName, target]) => {
          if (!Number.isFinite(target)) return;
          const joint = robot.joints?.[jointName];
          if (!joint?.setJointValue) return;
          next[jointName] = Number(target);
          joint.setJointValue(Number(target));
        });
        animJointRef.current = next;
      }
      clearTrail();
      const eff = endEffectorRef.current;
      if (eff) {
        const p = eff.getWorldPosition(tmpWorldRef.current).clone();
        pushTrailPoint(p, captureJointSnapshot());
      }
    } else {
      clearTrail();
    }
  }, [simMode, clearTrail, pushTrailPoint, captureJointSnapshot]);

  React.useEffect(() => {
    trailColorRef.current = trailColor;
    applyTrajectoryColor(trailColor);
  }, [trailColor, applyTrajectoryColor]);

  React.useEffect(() => {
    trailStyleRef.current = trailStyle;
    const canShow = modeRef.current === 'trajectory' && trailVisibleRef.current;
    if (trailLineRef.current) {
      trailLineRef.current.visible = canShow && trailStyle !== 'multi';
    }
    if (trailRainbowRef.current) {
      trailRainbowRef.current.visible = canShow;
    }
  }, [trailStyle]);

  React.useEffect(() => {
    trailVisibleRef.current = trailVisible;
    const canShow = modeRef.current === 'trajectory' && trailVisible;
    if (trailLineRef.current) {
      trailLineRef.current.visible = canShow && trailStyleRef.current !== 'multi';
    }
    if (trailRainbowRef.current) {
      trailRainbowRef.current.visible = canShow;
    }
    if (trailDotsRef.current) {
      trailDotsRef.current.visible = canShow;
    }
    if (trailTubeRef.current) {
      trailTubeRef.current.visible = canShow;
    }
    if (trailHeadRef.current) {
      trailHeadRef.current.visible = canShow;
    }
  }, [trailVisible]);

  React.useEffect(() => {
    clearTrail();
  }, [clearTrailSeq, clearTrail]);

  React.useEffect(() => {
    if (!exportTrailSeq) return;
    const frames = trailFramesRef.current || [];
    if (frames.length === 0) return;
    const payload = {
      schema: 'arm_traj_sequence_v2',
      frame: 'world',
      point_count: frames.length,
      exported_at: new Date().toISOString(),
      points: frames.map((f, i) => {
        const item = {
          i,
          x: Number(f?.pos?.x),
          y: Number(f?.pos?.y),
          z: Number(f?.pos?.z),
        };
        if (f?.joints && Object.keys(f.joints).length > 0) item.joints = f.joints;
        return item;
      }),
    };
    const blob = new Blob([`${JSON.stringify(payload, null, 2)}\n`], { type: 'application/json;charset=utf-8' });
    const now = new Date();
    const pad = (n) => String(n).padStart(2, '0');
    const ts = `${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
    const fileName = `arm_traj_sequence_${ts}.json`;
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  }, [exportTrailSeq]);

  React.useEffect(() => {
    replaySpeedRef.current = Math.max(0.05, Number(replaySpeed) || 1);
  }, [replaySpeed]);

  React.useEffect(() => {
    const raw = importedTrail?.points;
    if (!Array.isArray(raw) || raw.length < 2) return;
    const frames = raw
      .map((p) => {
        const x = Number(p?.x);
        const y = Number(p?.y);
        const z = Number(p?.z);
        if (!Number.isFinite(x) || !Number.isFinite(y) || !Number.isFinite(z)) return null;
        return {
          pos: new THREE.Vector3(x, y, z),
          joints: normalizeJointMap(p?.joints),
        };
      })
      .filter(Boolean);
    if (frames.length < 2) return;
    importedFramesRef.current = frames;
    replayRef.current = { active: false, frames: [], cursor: 0 };
    setReplayBusy(false);
    // Import stores sequence only; it doesn't auto-render full trajectory.
    // User triggers replay explicitly.
    if (trailHeadRef.current) trailHeadRef.current.position.copy(frames[0].pos);
  }, [importedTrail, normalizeJointMap, setReplayBusy]);

  React.useEffect(() => {
    if (!replaySeq) return;
    const frames = importedFramesRef.current || [];
    if (frames.length < 2) return;
    clearTrail();
    if (trailHeadRef.current) trailHeadRef.current.position.copy(frames[0].pos);
    applyJointMap(frames[0].joints);
    if (trailVisibleRef.current) pushTrailPoint(frames[0].pos, frames[0].joints);
    replayRef.current = { active: true, frames, cursor: 0 };
    setReplayBusy(true);
  }, [replaySeq, clearTrail, pushTrailPoint, applyJointMap, setReplayBusy]);

  React.useEffect(() => {
    if (!replayStopSeq) return;
    const replay = replayRef.current;
    if (!replay.active) return;
    replay.active = false;
    setReplayBusy(false);
  }, [replayStopSeq, setReplayBusy]);

  React.useEffect(() => {
    if (!replayFinishSeq) return;
    const replay = replayRef.current;
    if (!replay.active || replay.frames.length === 0) return;
    const lastFrame = replay.frames[replay.frames.length - 1];
    if (trailHeadRef.current) trailHeadRef.current.position.copy(lastFrame.pos);
    applyJointMap(lastFrame.joints);
    if (trailVisibleRef.current) pushTrailPoint(lastFrame.pos.clone(), lastFrame.joints);
    replay.cursor = replay.frames.length - 1;
    replay.active = false;
    setReplayBusy(false);
  }, [replayFinishSeq, applyJointMap, pushTrailPoint, setReplayBusy]);

  React.useEffect(() => {
    targetRef.current = jointTargets || {};
    if (modeRef.current !== 'direct') return;
    const robot = robotRef.current;
    if (!robot) return;
    Object.entries(targetRef.current).forEach(([jointName, rad]) => {
      if (!Number.isFinite(rad)) return;
      const j = robot.joints?.[jointName];
      if (j?.setJointValue) j.setJointValue(rad);
    });
  }, [jointTargets]);

  React.useEffect(() => {
    const host = hostRef.current;
    if (!host) return undefined;

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    renderer.setSize(host.clientWidth || 640, host.clientHeight || 400);
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    host.appendChild(renderer.domElement);

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0xf7fbff);

    const camera = new THREE.PerspectiveCamera(48, (host.clientWidth || 640) / (host.clientHeight || 400), 0.01, 20);
    camera.position.copy(DEFAULT_CAMERA_POS);
    cameraRef.current = camera;

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.copy(DEFAULT_TARGET);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.update();
    controlsRef.current = controls;

    const hemi = new THREE.HemisphereLight(0xffffff, 0x8ca6db, 1.0);
    scene.add(hemi);
    const dir = new THREE.DirectionalLight(0xffffff, 1.0);
    dir.position.set(2.0, 3.0, 1.6);
    scene.add(dir);
    const fill = new THREE.DirectionalLight(0xaec6ff, 0.5);
    fill.position.set(-1.2, 1.8, -1.4);
    scene.add(fill);

    const grid = new THREE.GridHelper(2.2, 24, 0x94ace1, 0xcad8f8);
    grid.position.y = -0.02;
    scene.add(grid);

    const trailGeometry = new LineGeometry();
    trailGeometry.setPositions(SAFE_ZERO_SEGMENT);
    const trailMaterial = new LineMaterial({
      color: 0xff0000,
      linewidth: 5.0,
      transparent: true,
      opacity: 1.0,
      toneMapped: false,
      depthTest: false,
      depthWrite: false,
      dashed: false,
    });
    trailMaterial.resolution.set(host.clientWidth || 640, host.clientHeight || 400);
    const trailLine = new Line2(trailGeometry, trailMaterial);
    trailLine.visible = modeRef.current === 'trajectory';
    trailLine.frustumCulled = false;
    trailLine.renderOrder = 1000;
    trailLine.computeLineDistances();
    scene.add(trailLine);
    trailLineRef.current = trailLine;
    const rainbowGeom = new THREE.BufferGeometry();
    rainbowGeom.setAttribute('position', new THREE.Float32BufferAttribute(SAFE_ZERO_SEGMENT, 3));
    rainbowGeom.setAttribute('color', new THREE.Float32BufferAttribute([1, 0, 0, 1, 0, 0], 3));
    const rainbowMat = new THREE.LineBasicMaterial({
      vertexColors: true,
      transparent: true,
      opacity: 0.98,
      depthTest: false,
      depthWrite: false,
      toneMapped: false,
    });
    const rainbowLine = new THREE.Line(rainbowGeom, rainbowMat);
    rainbowLine.visible = modeRef.current === 'trajectory';
    rainbowLine.frustumCulled = false;
    rainbowLine.renderOrder = 1002;
    scene.add(rainbowLine);
    trailRainbowRef.current = rainbowLine;
    const trailDots = new THREE.Points(
      new THREE.BufferGeometry().setFromPoints([]),
      new THREE.PointsMaterial({
        color: 0xff0000,
        size: 0.03,
        sizeAttenuation: true,
        transparent: true,
        opacity: 0.95,
        depthTest: false,
        depthWrite: false,
      }),
    );
    trailDots.visible = modeRef.current === 'trajectory';
    trailDots.frustumCulled = false;
    trailDots.renderOrder = 1000;
    scene.add(trailDots);
    trailDotsRef.current = trailDots;
    const trailTube = new THREE.Mesh(
      new THREE.BufferGeometry(),
      new THREE.MeshBasicMaterial({
        color: 0xff0000,
        transparent: true,
        opacity: 0.92,
        depthTest: false,
        depthWrite: false,
        toneMapped: false,
      }),
    );
    trailTube.visible = modeRef.current === 'trajectory';
    trailTube.frustumCulled = false;
    trailTube.renderOrder = 999;
    scene.add(trailTube);
    trailTubeRef.current = trailTube;

    applyTrajectoryColor(trailColorRef.current);

    const trailHead = new THREE.Mesh(
      new THREE.SphereGeometry(0.01, 12, 12),
      new THREE.MeshBasicMaterial({
        color: 0xffc93c,
        transparent: true,
        opacity: 0.98,
        depthTest: false,
        depthWrite: false,
      }),
    );
    trailHead.visible = modeRef.current === 'trajectory';
    trailHead.renderOrder = 1001;
    scene.add(trailHead);
    trailHeadRef.current = trailHead;

    const loader = new URDFLoader();
    loader.packages = { 'reBot-DevArm_description_fixend': '/resources/arm01' };
    loader.load(
      URDF_PATH,
      (robot) => {
        // ROS URDF is typically Z-up; this viewer uses Y-up.
        // Rotate to keep the base plate downward and the arm upright on the ground grid.
        robot.rotation.x = -Math.PI / 2;
        robot.position.set(0, 0.02, 0);
        scene.add(robot);
        robotRef.current = robot;
        endEffectorRef.current =
          robot.links?.end_link ||
          robot.links?.link6 ||
          robot.links?.tool0 ||
          robot.links?.ee_link ||
          null;
        if (!endEffectorRef.current) {
          robot.traverse((obj) => {
            if (endEffectorRef.current) return;
            if (obj?.name && /end|ee|tool/i.test(obj.name)) endEffectorRef.current = obj;
          });
        }
        if (!endEffectorRef.current) {
          robot.traverse((obj) => {
            if (!obj?.isObject3D) return;
            if (!obj.children || obj.children.length === 0) endEffectorRef.current = obj;
          });
        }
        if (robot?.joints) {
          const next = {};
          Object.entries(targetRef.current || {}).forEach(([jointName, targetRaw]) => {
            const target = Number(targetRaw);
            if (!Number.isFinite(target)) return;
            const j = robot.joints?.[jointName];
            if (!j?.setJointValue) return;
            j.setJointValue(target);
            next[jointName] = target;
          });
          animJointRef.current = next;
        }
        robot.updateMatrixWorld(true);
        clearTrail();
        if (modeRef.current === 'trajectory' && endEffectorRef.current) {
          pushTrailPoint(endEffectorRef.current.getWorldPosition(tmpWorldRef.current).clone(), captureJointSnapshot());
        }
        setStatus('ready');
      },
      undefined,
      () => {
        setStatus('error');
      },
    );

    const smoothStepTrajectory = (dtSec) => {
      const robot = robotRef.current;
      if (!robot?.joints) return;
      const targets = targetRef.current || {};
      const nextState = { ...animJointRef.current };
      Object.entries(targets).forEach(([jointName, targetRaw]) => {
        const target = Number(targetRaw);
        if (!Number.isFinite(target)) return;
        const j = robot.joints?.[jointName];
        if (!j?.setJointValue) return;
        const current = Number.isFinite(nextState[jointName]) ? nextState[jointName] : target;
        const diff = target - current;
        if (Math.abs(diff) < 1e-5) {
          nextState[jointName] = target;
          j.setJointValue(target);
          return;
        }
        const eased = diff * 0.18;
        const maxStep = Math.max(0.006, dtSec * 1.0);
        const step = Math.max(-maxStep, Math.min(maxStep, eased));
        const next = current + step;
        nextState[jointName] = next;
        j.setJointValue(next);
      });
      animJointRef.current = nextState;
    };

      const sampleTrail = () => {
      if (modeRef.current !== 'trajectory') return;
      const eff = endEffectorRef.current;
      const line = trailLineRef.current;
      if (!eff || !line) return;
      robotRef.current?.updateMatrixWorld(true);
      const p = eff.getWorldPosition(tmpWorldRef.current).clone();
      if (trailHeadRef.current) trailHeadRef.current.position.copy(p);
      const raw = trailPointsRef.current;
      const last = raw[raw.length - 1];
      if (!last || last.distanceToSquared(p) >= TRAIL_SAMPLE_MIN_DIST * TRAIL_SAMPLE_MIN_DIST) {
        pushTrailPoint(p, captureJointSnapshot());
      }
    };

    const clock = new THREE.Clock();
    let raf = 0;
    const renderLoop = () => {
      const dt = clock.getDelta();
      if (modeRef.current === 'trajectory') {
        if (trailHeadRef.current) trailHeadRef.current.visible = trailVisibleRef.current;
        const replay = replayRef.current;
        if (replay.active && trailHeadRef.current && replay.frames.length >= 2) {
          replay.cursor += dt * 22 * replaySpeedRef.current;
          const idx = Math.floor(replay.cursor);
          if (idx >= replay.frames.length - 1) {
            const lastFrame = replay.frames[replay.frames.length - 1];
            trailHeadRef.current.position.copy(lastFrame.pos);
            applyJointMap(lastFrame.joints);
            if (trailVisibleRef.current) pushTrailPoint(lastFrame.pos.clone(), lastFrame.joints);
            replay.active = false;
            setReplayBusy(false);
          } else {
            const t = replay.cursor - idx;
            const a = replay.frames[idx];
            const b = replay.frames[idx + 1];
            trailHeadRef.current.position.lerpVectors(a.pos, b.pos, t);
            const jointsNow = interpolateJointMap(a.joints, b.joints, t);
            applyJointMap(jointsNow);
            if (trailVisibleRef.current) pushTrailPoint(trailHeadRef.current.position.clone(), jointsNow);
          }
        } else {
          smoothStepTrajectory(dt);
          sampleTrail();
        }
      } else if (trailHeadRef.current) {
        trailHeadRef.current.visible = false;
      }
      controls.update();
      renderer.render(scene, camera);
      raf = window.requestAnimationFrame(renderLoop);
    };
    renderLoop();

    const resize = () => {
      if (!host) return;
      const w = host.clientWidth || 640;
      const h = host.clientHeight || 400;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
      const mat = trailLineRef.current?.material;
      if (mat?.resolution) mat.resolution.set(w, h);
    };
    window.addEventListener('resize', resize);

    return () => {
      window.removeEventListener('resize', resize);
      window.cancelAnimationFrame(raf);
      controls.dispose();
      renderer.dispose();
      scene.traverse((obj) => {
        if (obj.geometry) obj.geometry.dispose?.();
        if (obj.material) {
          if (Array.isArray(obj.material)) obj.material.forEach((m) => m.dispose?.());
          else obj.material.dispose?.();
        }
      });
      if (renderer.domElement.parentElement === host) host.removeChild(renderer.domElement);
      robotRef.current = null;
      cameraRef.current = null;
      controlsRef.current = null;
      trailLineRef.current = null;
      trailRainbowRef.current = null;
      trailDotsRef.current = null;
      trailTubeRef.current = null;
      trailHeadRef.current = null;
      endEffectorRef.current = null;
      trailPointsRef.current = [];
      trailFramesRef.current = [];
      importedFramesRef.current = [];
      replayRef.current = { active: false, frames: [], cursor: 0 };
      setReplayBusy(false);
    };
  }, [
    pushTrailPoint,
    clearTrail,
    applyTrajectoryColor,
    captureJointSnapshot,
    applyJointMap,
    interpolateJointMap,
    setReplayBusy,
  ]);

  React.useEffect(() => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) return;
    camera.position.copy(DEFAULT_CAMERA_POS);
    controls.target.copy(DEFAULT_TARGET);
    controls.update();
    clearTrail();
  }, [resetViewSeq, clearTrail]);

  return (
    <div className="armUrdfViewerShell">
      <div ref={hostRef} className="armUrdfViewer" />
      {status !== 'ready' && (
        <div className="armUrdfOverlay">{status === 'error' ? 'URDF load failed' : 'Loading URDF...'}</div>
      )}
    </div>
  );
}
