import { lazy, Suspense } from 'react'

const Scene3D = lazy(() => import('./Scene3D'))

export default function Scene3DLazy() {
  return (
    <Suspense fallback={<div style={{ width: '100%', height: '100%' }} />}>
      <Scene3D />
    </Suspense>
  )
}
