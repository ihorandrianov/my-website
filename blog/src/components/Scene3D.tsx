import { Canvas, useFrame } from '@react-three/fiber'
import { useGLTF, OrbitControls } from '@react-three/drei'
import { EffectComposer, Bloom } from '@react-three/postprocessing'
import { useRef, useEffect, type JSX } from 'react'
import * as THREE from 'three'

type RamenGLTF = {
  nodes: {
    Ramen_soup_neon_Ramen_soup_neon_0: THREE.Mesh
  }
  materials: {
    Ramen_soup_neon: THREE.MeshStandardMaterial
  }
}


export function RamenSoup(props: JSX.IntrinsicElements['group']) {
  const { nodes, materials } = useGLTF('/models/ramen_soup.gltf') as unknown as RamenGLTF
  const meshRef = useRef<THREE.Mesh>(null)

  useEffect(() => {
    if (materials.Ramen_soup_neon) {
      materials.Ramen_soup_neon.emissive = new THREE.Color('#c9a87c')
    }
  }, [materials])

  useFrame((state) => {
    if (meshRef.current) {
      const t = state.clock.elapsedTime
      // Neon flicker pattern - more frequent flickers
      const cycle = Math.floor(t * 0.5) % 8
      let isOff = false

      if (cycle === 2 || cycle === 5) {
        // Quick double flicker
        const phase = (t * 0.5 % 1) * 8
        isOff = phase < 0.3 || (phase > 0.5 && phase < 0.7)
      } else if (cycle === 7) {
        // Longer off period
        const phase = (t * 0.5 % 1) * 8
        isOff = phase < 0.8
      }

      const material = meshRef.current.material as THREE.MeshStandardMaterial
      material.emissiveIntensity = isOff ? 0.05 : 1
    }
  })

  return (
    <group {...props} dispose={null}>
      <group scale={0.1}>
        <mesh
          ref={meshRef}
          castShadow
          receiveShadow
          geometry={nodes.Ramen_soup_neon_Ramen_soup_neon_0.geometry}
          material={materials.Ramen_soup_neon}
          scale={80.777}
          rotation={[-Math.PI / 2, 0, 0]}
        />
      </group>
    </group>
  )
}

useGLTF.preload('/models/ramen_soup.gltf')
export default function Scene3D() {
  return (
    <Canvas style={{ height: '100%', width: '100%' }} camera={{ position: [0, 10, 20], fov: 60 }}>

      {/* Noir lighting setup */}
      <ambientLight intensity={0.05} />
      <spotLight
        position={[-10, 20, 5]}
        angle={0.4}
        penumbra={0.8}
        intensity={1.5}
        color="#ffeedd"
      />
      <pointLight position={[10, 5, -10]} intensity={0.3} color="#aaaaff" />

      <RamenSoup position={[0, 10, 0]} />
      <OrbitControls target={[0, 10, 0]} enableZoom={false} enableRotate={false} />
      <EffectComposer>
        <Bloom intensity={1.5} luminanceThreshold={0.1} luminanceSmoothing={0.2} radius={0.6} />
      </EffectComposer>
    </Canvas>
  )
}
