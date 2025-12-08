import { Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider, useAuth } from './contexts/AuthContext'
import { canAccess } from './utils/permissions'
import Login from './pages/Login'
import Dashboard from './pages/Dashboard'
import ArtifactGallery from './pages/ArtifactGallery'
import UploadArtifact from './pages/UploadArtifact'
import UserManagement from './pages/UserManagement'
import AuditLogs from './pages/AuditLogs'
import ChangePassword from './pages/ChangePassword'
import Layout from './components/Layout'

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth()
  return isAuthenticated ? <>{children}</> : <Navigate to="/login" replace />
}

function AppRoutes() {
  const { user } = useAuth()

  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route
        path="/"
        element={
          <ProtectedRoute>
            <Layout />
          </ProtectedRoute>
        }
      >
        <Route index element={<Dashboard />} />
        <Route path="gallery" element={<ArtifactGallery />} />
        <Route path="upload" element={<UploadArtifact />} />
        {canAccess(user?.role, 'user-management') && (
          <Route path="users" element={<UserManagement />} />
        )}
        {canAccess(user?.role, 'audit-logs') && (
          <Route path="audit-logs" element={<AuditLogs />} />
        )}
        <Route path="change-password" element={<ChangePassword />} />
      </Route>
    </Routes>
  )
}

function App() {
  return (
    <AuthProvider>
      <AppRoutes />
    </AuthProvider>
  )
}

export default App

