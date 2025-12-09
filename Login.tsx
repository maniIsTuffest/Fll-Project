// src/pages/Login.tsx
import './Register.css'
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../contexts/AuthContext'
import { api } from '../services/api'
import './Login.css'

export default function Login() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const { login } = useAuth()
  const navigate = useNavigate()

  
  const registerButton = () => {
    navigate('/signup') 
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)

    try {
      const response = await api.post('/auth/login', { username, password })
      if (response.data) {
        await login(username, password)
        navigate('/')
      } else {
        setError('Invalid username or password')
      }
    } catch (err: any) {
      if (err.response?.status === 401) setError('Invalid username or password')
      else if (err.response?.status === 404)
        setError('Authentication service unavailable. Please contact administrator.')
      else setError('Login failed. Please check your credentials and try again.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      <div>
        <button
          type="button" 
          disabled={loading}
          className="register-button"
          onClick={registerButton}
        >
          {loading ? 'Registering...' : 'Register'}
        </button>
      </div>

      <div className="login-container">
        <div className="login-box">
          <h1>🏺 ArtiQuest</h1>
          <h2>Login</h2>
          <form onSubmit={handleSubmit}>
            <div className="form-group">
              <label htmlFor="username">Username</label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                required
                autoFocus
              />
            </div>
            <div className="form-group">
              <label htmlFor="password">Password</label>
              <input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
              />
            </div>
            {error && <div className="error-message">{error}</div>}
            <button type="submit" disabled={loading} className="login-button">
              {loading ? 'Logging in...' : 'Login'}
            </button>
          </form>
        </div>
      </div>
    </>
  )
}
