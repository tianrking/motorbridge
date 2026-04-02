import React from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import URDFLoader from 'urdf-loader';

const URDF_PATH = '/resources/arm01/urdf/reBot-DevArm_description_fixend.urdf';

export function ArmUrdfViewer({ jointTargets }) {
  const hostRef = React.useRef(null);
  const robotRef = React.useRef(null);
  const [status, setStatus] = React.useState('loading');

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
    camera.position.set(1.15, 0.95, 1.2);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.set(0, 0.35, 0);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.update();

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
        setStatus('ready');
      },
      undefined,
      () => {
        setStatus('error');
      },
    );

    let raf = 0;
    const renderLoop = () => {
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
    };
  }, []);

  React.useEffect(() => {
    const robot = robotRef.current;
    if (!robot || !jointTargets) return;
    Object.entries(jointTargets).forEach(([jointName, rad]) => {
      if (!Number.isFinite(rad)) return;
      const j = robot.joints?.[jointName];
      if (j?.setJointValue) j.setJointValue(rad);
    });
  }, [jointTargets]);

  return (
    <div className="armUrdfViewerShell">
      <div ref={hostRef} className="armUrdfViewer" />
      {status !== 'ready' && (
        <div className="armUrdfOverlay">{status === 'error' ? 'URDF load failed' : 'Loading URDF...'}</div>
      )}
    </div>
  );
}
