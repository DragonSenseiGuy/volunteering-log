<script lang="ts">
  import { onMount } from 'svelte';
  import Database from '@tauri-apps/plugin-sql';

  interface Profile {
    id: number;
    name: string;
  }

  interface VolunteerEntry {
    id: number;
    profile_id: number;
    place: string;
    date: string;
    hours: number;
    notes: string;
  }

  let db: Database | null = null;
  let profiles: Profile[] = [];
  let currentProfile: Profile | null = null;
  let entries: VolunteerEntry[] = [];
  let place = '';
  let date = new Date().toISOString().split('T')[0];
  let hours: number | string = '';
  let notes = '';
  let editingId: number | null = null;
  
  let activeTab: 'add' | 'log' = 'add';
  let selectedYear: number | 'all' = 'all';
  let currentPage = 1;
  const perPage = 10;

  let showProfileModal = true;
  let newProfileName = '';

  onMount(async () => {
    try {
      db = await Database.load('sqlite:volunteer.db');
      
      await db.execute(`
        CREATE TABLE IF NOT EXISTS profiles (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL UNIQUE
        )
      `);
      
      await db.execute(`
        CREATE TABLE IF NOT EXISTS entries (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          profile_id INTEGER NOT NULL,
          place TEXT NOT NULL,
          date TEXT NOT NULL,
          hours REAL NOT NULL,
          notes TEXT DEFAULT '',
          FOREIGN KEY (profile_id) REFERENCES profiles(id)
        )
      `);
      
      await loadProfiles();
      
      if (profiles.length === 0) {
        showProfileModal = true;
      } else {
        currentProfile = profiles[0];
        await loadEntries();
      }
    } catch (e) {
      console.error('Database init error:', e);
      alert('Database error: ' + e);
    }
  });

  async function loadProfiles() {
    if (!db) return;
    profiles = await db.select<Profile[]>('SELECT * FROM profiles ORDER BY name');
  }

  async function loadEntries() {
    if (!db || !currentProfile) return;
    entries = await db.select<VolunteerEntry[]>(
      'SELECT * FROM entries WHERE profile_id = ? ORDER BY date DESC',
      [currentProfile.id]
    );
  }

  async function createProfile() {
    if (!newProfileName.trim()) {
      alert('Please enter a name');
      return;
    }
    if (!db) {
      alert('Database not loaded yet');
      return;
    }
    
    try {
      await db.execute('INSERT INTO profiles (name) VALUES (?)', [newProfileName.trim()]);
      await loadProfiles();
      currentProfile = profiles.find(p => p.name === newProfileName.trim()) || profiles[0];
      newProfileName = '';
      showProfileModal = false;
      await loadEntries();
    } catch (e) {
      console.error('Create profile error:', e);
      alert('Error creating profile: ' + e);
    }
  }

  async function switchProfile(profile: Profile) {
    currentProfile = profile;
    currentPage = 1;
    selectedYear = 'all';
    await loadEntries();
  }

  async function deleteProfile(profile: Profile) {
    if (!db) return;
    if (!confirm(`Delete profile "${profile.name}" and all their entries?`)) return;
    
    await db.execute('DELETE FROM entries WHERE profile_id = ?', [profile.id]);
    await db.execute('DELETE FROM profiles WHERE id = ?', [profile.id]);
    await loadProfiles();
    
    if (profiles.length === 0) {
      currentProfile = null;
      entries = [];
      showProfileModal = true;
    } else if (currentProfile?.id === profile.id) {
      currentProfile = profiles[0];
      await loadEntries();
    }
  }

  function getYears(): number[] {
    const years = new Set(entries.map(e => new Date(e.date).getFullYear()));
    return Array.from(years).sort((a, b) => b - a);
  }

  function getFilteredEntries(): VolunteerEntry[] {
    let filtered = entries;
    if (selectedYear !== 'all') {
      filtered = entries.filter(e => new Date(e.date).getFullYear() === selectedYear);
    }
    return filtered.sort((a, b) => b.date.localeCompare(a.date));
  }

  function getPaginatedEntries(): VolunteerEntry[] {
    const filtered = getFilteredEntries();
    const start = (currentPage - 1) * perPage;
    return filtered.slice(start, start + perPage);
  }

  function getTotalPages(): number {
    return Math.ceil(getFilteredEntries().length / perPage);
  }

  function getFilteredHours(): number {
    return getFilteredEntries().reduce((sum, e) => sum + e.hours, 0);
  }

  function getTotalHours(): number {
    return entries.reduce((sum, e) => sum + e.hours, 0);
  }

  async function handleSubmit() {
    if (!db || !currentProfile || !place || !date || !hours) return;
    
    const hoursNum = typeof hours === 'string' ? parseFloat(hours) : hours;
    
    if (editingId !== null) {
      await db.execute(
        'UPDATE entries SET place = ?, date = ?, hours = ?, notes = ? WHERE id = ?',
        [place, date, hoursNum, notes, editingId]
      );
      editingId = null;
    } else {
      await db.execute(
        'INSERT INTO entries (profile_id, place, date, hours, notes) VALUES (?, ?, ?, ?, ?)',
        [currentProfile.id, place, date, hoursNum, notes]
      );
    }
    
    await loadEntries();
    resetForm();
  }

  function resetForm() {
    place = '';
    date = new Date().toISOString().split('T')[0];
    hours = '';
    notes = '';
    editingId = null;
  }

  function editEntry(entry: VolunteerEntry) {
    place = entry.place;
    date = entry.date;
    hours = entry.hours;
    notes = entry.notes;
    editingId = entry.id;
    activeTab = 'add';
  }

  async function deleteEntry(id: number) {
    if (!db) return;
    await db.execute('DELETE FROM entries WHERE id = ?', [id]);
    await loadEntries();
    if (getPaginatedEntries().length === 0 && currentPage > 1) {
      currentPage--;
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr + 'T00:00:00').toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  function handleYearChange() {
    currentPage = 1;
  }
</script>

{#if showProfileModal}
  <div class="modal-overlay">
    <div class="modal">
      <h2>👤 {profiles.length === 0 ? 'Create Your Profile' : 'Add New Profile'}</h2>
      <form on:submit|preventDefault={createProfile}>
        <input
          type="text"
          bind:value={newProfileName}
          placeholder="Enter name"
          required
        />
        <div class="modal-actions">
          {#if profiles.length > 0}
            <button type="button" class="btn-secondary" on:click={() => showProfileModal = false}>Cancel</button>
          {/if}
          <button type="submit" class="btn-primary" on:click={createProfile}>Create Profile</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<main>
  <header>
    <h1>📋 Volunteering Log</h1>
    <div class="total-hours">
      Total: <strong>{getTotalHours().toFixed(1)}</strong> hours
    </div>
  </header>

  {#if currentProfile}
    <div class="profile-bar">
      <div class="profile-selector">
        <span class="profile-label">👤</span>
        <select bind:value={currentProfile} on:change={() => currentProfile && switchProfile(currentProfile)}>
          {#each profiles as profile}
            <option value={profile}>{profile.name}</option>
          {/each}
        </select>
      </div>
      <div class="profile-actions">
        <button class="btn-small primary" on:click={() => showProfileModal = true}>+ Add</button>
        {#if profiles.length > 1}
          <button class="btn-small delete" on:click={() => currentProfile && deleteProfile(currentProfile)}>🗑️</button>
        {/if}
      </div>
    </div>
  {/if}

  <nav class="tabs">
    <button 
      class="tab" 
      class:active={activeTab === 'add'} 
      on:click={() => activeTab = 'add'}
    >
      {editingId ? 'Edit Entry' : 'Add Entry'}
    </button>
    <button 
      class="tab" 
      class:active={activeTab === 'log'} 
      on:click={() => activeTab = 'log'}
    >
      View Log
    </button>
  </nav>

  {#if activeTab === 'add'}
    <section class="tab-content">
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="place">Place</label>
          <input
            type="text"
            id="place"
            bind:value={place}
            placeholder="Organization name"
            required
          />
        </div>
        <div class="form-group">
          <label for="date">📅 Date (tap to change)</label>
          <input
            type="date"
            id="date"
            bind:value={date}
            required
          />
        </div>
        <div class="form-group">
          <label for="hours">⏱️ Hours</label>
          <input
            type="number"
            id="hours"
            bind:value={hours}
            placeholder="0"
            step="0.5"
            min="0"
            required
          />
        </div>
        <div class="form-group">
          <label for="notes">Notes</label>
          <textarea
            id="notes"
            bind:value={notes}
            placeholder="What did you do? (optional)"
            rows="3"
          ></textarea>
        </div>
        <div class="form-actions">
          {#if editingId}
            <button type="button" class="btn-secondary" on:click={resetForm}>Cancel</button>
          {/if}
          <button type="submit" class="btn-primary">
            {editingId ? 'Update Entry' : 'Add Entry'}
          </button>
        </div>
      </form>
    </section>
  {:else}
    <section class="tab-content">
      <div class="log-header">
        <div class="year-filter">
          <label for="year">Year:</label>
          <select id="year" bind:value={selectedYear} on:change={handleYearChange}>
            <option value="all">All Years</option>
            {#each getYears() as year}
              <option value={year}>{year}</option>
            {/each}
          </select>
        </div>
        {#if selectedYear !== 'all'}
          <div class="year-hours">
            {selectedYear}: <strong>{getFilteredHours().toFixed(1)}</strong> hrs
          </div>
        {/if}
      </div>

      {#if entries.length === 0}
        <p class="empty-state">No volunteer hours logged yet. Add your first entry!</p>
      {:else if getFilteredEntries().length === 0}
        <p class="empty-state">No entries for {selectedYear}.</p>
      {:else}
        <div class="entries-list">
          {#each getPaginatedEntries() as entry}
            <div class="entry-card">
              <div class="entry-header">
                <span class="entry-place">{entry.place}</span>
                <span class="entry-hours">{entry.hours} hrs</span>
              </div>
              <div class="entry-date">{formatDate(entry.date)}</div>
              {#if entry.notes}
                <div class="entry-notes">{entry.notes}</div>
              {/if}
              <div class="entry-actions">
                <button class="btn-icon" on:click={() => editEntry(entry)} title="Edit">✏️ Edit</button>
                <button class="btn-icon delete" on:click={() => deleteEntry(entry.id)} title="Delete">🗑️ Delete</button>
              </div>
            </div>
          {/each}
        </div>

        {#if getTotalPages() > 1}
          <div class="pagination">
            <button 
              class="btn-page" 
              disabled={currentPage === 1}
              on:click={() => currentPage--}
            >
              ← Prev
            </button>
            <span class="page-info">
              {currentPage} / {getTotalPages()}
            </span>
            <button 
              class="btn-page" 
              disabled={currentPage === getTotalPages()}
              on:click={() => currentPage++}
            >
              Next →
            </button>
          </div>
        {/if}
      {/if}
    </section>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background: #f5f7fa;
    color: #333;
    -webkit-font-smoothing: antialiased;
  }

  main {
    max-width: 600px;
    margin: 0 auto;
    padding: 16px;
    padding-top: env(safe-area-inset-top, 16px);
    padding-bottom: env(safe-area-inset-bottom, 16px);
    box-sizing: border-box;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 2px solid #e1e5eb;
    gap: 12px;
  }

  h1 {
    margin: 0;
    font-size: 1.4rem;
    color: #1a1a2e;
    flex-shrink: 1;
    min-width: 0;
  }

  .total-hours {
    background: #4a6cf7;
    color: white;
    padding: 8px 12px;
    border-radius: 8px;
    font-size: 0.9rem;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .profile-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: white;
    padding: 10px 14px;
    border-radius: 10px;
    margin-bottom: 12px;
    box-shadow: 0 1px 4px rgba(0,0,0,0.06);
  }

  .profile-selector {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .profile-label {
    font-size: 1.2rem;
  }

  .profile-selector select {
    padding: 6px 10px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    font-weight: 600;
    background: white;
    cursor: pointer;
  }

  .profile-actions {
    display: flex;
    gap: 8px;
  }

  .btn-small {
    padding: 6px 12px;
    border: none;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    background: #e1e5eb;
    color: #555;
  }

  .btn-small:hover {
    background: #d1d5db;
  }

  .btn-small.primary {
    background: #4a6cf7;
    color: white;
  }

  .btn-small.primary:hover {
    background: #3a5ce5;
  }

  .btn-small.delete {
    background: #fed7d7;
    color: #c53030;
  }

  .btn-small.delete:hover {
    background: #feb2b2;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
  }

  .modal {
    background: white;
    padding: 24px;
    border-radius: 16px;
    width: 100%;
    max-width: 320px;
    box-shadow: 0 10px 40px rgba(0,0,0,0.2);
  }

  .modal h2 {
    margin: 0 0 16px 0;
    font-size: 1.3rem;
    text-align: center;
  }

  .modal input {
    width: 100%;
    padding: 12px;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 16px;
    box-sizing: border-box;
    margin-bottom: 16px;
  }

  .modal input:focus {
    outline: none;
    border-color: #4a6cf7;
  }

  .modal-actions {
    display: flex;
    gap: 10px;
    justify-content: center;
  }

  .tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 0;
  }

  .tab {
    flex: 1;
    padding: 12px 16px;
    border: none;
    background: #e1e5eb;
    color: #555;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    border-radius: 8px 8px 0 0;
    transition: background 0.2s, color 0.2s;
  }

  .tab:hover {
    background: #d1d5db;
  }

  .tab.active {
    background: white;
    color: #4a6cf7;
  }

  .tab-content {
    background: white;
    padding: 20px;
    border-radius: 0 0 12px 12px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
  }

  .log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #e1e5eb;
    flex-wrap: wrap;
    gap: 10px;
  }

  .year-filter {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .year-filter label {
    font-weight: 600;
    color: #555;
    font-size: 0.9rem;
  }

  .year-filter select {
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    background: white;
    cursor: pointer;
  }

  .year-hours {
    background: #e8f4e8;
    color: #2d5a2d;
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 0.85rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
  }

  label {
    font-size: 0.875rem;
    font-weight: 600;
    color: #555;
    margin-bottom: 6px;
  }

  input, textarea {
    padding: 12px;
    border: 1px solid #ddd;
    border-radius: 8px;
    font-size: 16px;
    transition: border-color 0.2s, box-shadow 0.2s;
    box-sizing: border-box;
    width: 100%;
    background: white;
    -webkit-appearance: none;
    appearance: none;
  }

  input[type="date"] {
    cursor: pointer;
    background: #f8f9fb;
    border: 2px solid #4a6cf7;
  }

  input:focus, textarea:focus {
    outline: none;
    border-color: #4a6cf7;
    box-shadow: 0 0 0 3px rgba(74, 108, 247, 0.15);
  }

  textarea {
    resize: vertical;
    font-family: inherit;
  }

  .form-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }

  button {
    padding: 12px 20px;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.1s, box-shadow 0.2s;
  }

  button:active {
    transform: scale(0.98);
  }

  .btn-primary {
    background: #4a6cf7;
    color: white;
  }

  .btn-primary:hover {
    background: #3a5ce5;
    box-shadow: 0 4px 12px rgba(74, 108, 247, 0.3);
  }

  .btn-secondary {
    background: #e1e5eb;
    color: #555;
  }

  .btn-secondary:hover {
    background: #d1d5db;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #888;
    font-size: 1rem;
  }

  .entries-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .entry-card {
    background: #f8f9fb;
    border-radius: 10px;
    padding: 14px;
    border: 1px solid #e1e5eb;
  }

  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .entry-place {
    font-weight: 600;
    font-size: 1rem;
    color: #1a1a2e;
  }

  .entry-hours {
    font-weight: 700;
    color: #4a6cf7;
    font-size: 1rem;
  }

  .entry-date {
    color: #666;
    font-size: 0.85rem;
    margin-bottom: 6px;
  }

  .entry-notes {
    color: #555;
    font-size: 0.9rem;
    margin-bottom: 10px;
    line-height: 1.4;
  }

  .entry-actions {
    display: flex;
    gap: 12px;
    padding-top: 10px;
    border-top: 1px solid #e1e5eb;
  }

  .btn-icon {
    padding: 8px 14px;
    background: #e1e5eb;
    font-size: 0.85rem;
    border-radius: 6px;
    color: #555;
  }

  .btn-icon:hover {
    background: #d1d5db;
  }

  .btn-icon.delete {
    color: #c53030;
  }

  .btn-icon.delete:hover {
    background: #fed7d7;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid #e1e5eb;
  }

  .btn-page {
    padding: 10px 16px;
    background: #e1e5eb;
    color: #555;
    font-size: 0.9rem;
  }

  .btn-page:hover:not(:disabled) {
    background: #d1d5db;
  }

  .btn-page:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-info {
    color: #666;
    font-size: 0.9rem;
  }

  @media (min-width: 768px) {
    main {
      padding: 24px;
    }

    h1 {
      font-size: 1.75rem;
    }

    .total-hours {
      padding: 8px 16px;
      font-size: 1rem;
    }

    .tab-content {
      padding: 24px;
    }
  }
</style>
