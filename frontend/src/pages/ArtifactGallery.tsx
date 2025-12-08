import { useEffect, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { artifactApi } from '../services/api'
import { Artifact } from '../types'
import ArtifactCard from '../components/ArtifactCard'
import ArtifactModal from '../components/ArtifactModal'
import './ArtifactGallery.css'

export default function ArtifactGallery() {
  const [artifacts, setArtifacts] = useState<Artifact[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedArtifact, setSelectedArtifact] = useState<Artifact | null>(null)
  const [searchParams, setSearchParams] = useSearchParams()

  useEffect(() => {
    const query = searchParams.get('q') || ''
    setSearchQuery(query)
    loadArtifacts(query)
  }, [searchParams])

  const loadArtifacts = async (query: string = '') => {
    setLoading(true)
    try {
      let data
      if (query.trim()) {
        try {
          data = await artifactApi.search(query)
        } catch (searchError: any) {
          // If search endpoint fails, fallback to getAll and filter client-side
          console.warn('Search endpoint failed, falling back to client-side filtering:', searchError)
          const allArtifacts = await artifactApi.getAll()
          const queryLower = query.toLowerCase()
          data = allArtifacts.filter((artifact: any) => 
            artifact.name?.toLowerCase().includes(queryLower) ||
            artifact.description?.toLowerCase().includes(queryLower) ||
            artifact.tags?.some((tag: string) => tag.toLowerCase().includes(queryLower))
          )
        }
      } else {
        data = await artifactApi.getAll()
      }
      setArtifacts(data)
    } catch (error) {
      console.error('Failed to load artifacts:', error)
      setArtifacts([])
    } finally {
      setLoading(false)
    }
  }

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    if (searchQuery) {
      setSearchParams({ q: searchQuery })
    } else {
      setSearchParams({})
    }
  }

  if (loading) {
    return <div className="gallery-loading">Loading artifacts...</div>
  }

  return (
    <div className="gallery">
      <div className="gallery-header">
        <h1>🏛️ Artifact Archive</h1>
        <form onSubmit={handleSearch} className="search-form">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search by name, description, material, or tags"
            className="search-input"
          />
          <button type="submit" className="search-button">
            🔎 Search
          </button>
        </form>
      </div>

      <div className="gallery-stats">
        <div className="stat">
          <strong>Total Artifacts:</strong> {artifacts.length}
        </div>
        {searchQuery && (
          <div className="stat">
            <strong>Search Results:</strong> {artifacts.length}
          </div>
        )}
      </div>

      {artifacts.length === 0 ? (
        <div className="empty-state">
          <p>📭 No artifacts found. Start by uploading one!</p>
        </div>
      ) : (
        <div className="gallery-grid">
          {artifacts.map((artifact) => (
            <ArtifactCard
              key={artifact.id}
              artifact={artifact}
              onClick={() => setSelectedArtifact(artifact)}
            />
          ))}
        </div>
      )}

      {selectedArtifact && (
        <ArtifactModal
          artifact={selectedArtifact}
          onClose={() => setSelectedArtifact(null)}
          onUpdate={loadArtifacts}
        />
      )}
    </div>
  )
}

