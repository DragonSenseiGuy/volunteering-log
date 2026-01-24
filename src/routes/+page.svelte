<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface VolunteerEntry {
    id: string;
    place: string;
    date: string;
    hours: number;
    notes: string;
  }

  let entries: VolunteerEntry[] = [];
  let place = '';
  let date = new Date().toISOString().split('T')[0];
  let hours: number | string = '';
  let notes = '';
  let editingId: string | null = null;
  
  let activeTab: 'add' | 'log' = 'add';
  let selectedYear: number | 'all' = 'all';
  let currentPage = 1;
  const perPage = 10;

  onMount(async () => {
    entries = await invoke('get_entries');
  });

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
    if (!place || !date || !hours) return;
    
    const hoursNum = typeof hours === 'string' ? parseFloat(hours) : hours;
    
    if (editingId) {
      entries = await invoke('update_entry', {
        id: editingId,
        place,
        date,
        hours: hoursNum,
        notes
      });
      editingId = null;
    } else {
      entries = await invoke('add_entry', {
        place,
        date,
        hours: hoursNum,
        notes
      });
    }
    
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

  async function deleteEntry(id: string) {
    entries = await invoke('delete_entry', { id });
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

<main>
  <header>
    <h1>📋 Volunteering Log</h1>
    <div class="total-hours">
      Total: <strong>{getTotalHours().toFixed(1)}</strong> hours
    </div>
  </header>

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
        <div class="form-row">
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
            <label for="date">Date</label>
            <input
              type="date"
              id="date"
              bind:value={date}
              required
            />
          </div>
          <div class="form-group small">
            <label for="hours">Hours</label>
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
        <div class="year-hours">
          {#if selectedYear !== 'all'}
            {selectedYear} Total: <strong>{getFilteredHours().toFixed(1)}</strong> hours
          {/if}
        </div>
      </div>

      {#if entries.length === 0}
        <p class="empty-state">No volunteer hours logged yet. Add your first entry!</p>
      {:else if getFilteredEntries().length === 0}
        <p class="empty-state">No entries for {selectedYear}.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>Date</th>
              <th>Place</th>
              <th>Hours</th>
              <th>Notes</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each getPaginatedEntries() as entry}
              <tr>
                <td class="date">{formatDate(entry.date)}</td>
                <td class="place">{entry.place}</td>
                <td class="hours">{entry.hours}</td>
                <td class="notes">{entry.notes || '-'}</td>
                <td class="actions">
                  <button class="btn-icon" on:click={() => editEntry(entry)} title="Edit">✏️</button>
                  <button class="btn-icon" on:click={() => deleteEntry(entry.id)} title="Delete">🗑️</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>

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
              Page {currentPage} of {getTotalPages()}
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
  }

  main {
    max-width: 900px;
    margin: 0 auto;
    padding: 24px;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 2px solid #e1e5eb;
  }

  h1 {
    margin: 0;
    font-size: 1.75rem;
    color: #1a1a2e;
  }

  .total-hours {
    background: #4a6cf7;
    color: white;
    padding: 8px 16px;
    border-radius: 8px;
    font-size: 1rem;
  }

  .tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 0;
  }

  .tab {
    padding: 12px 24px;
    border: none;
    background: #e1e5eb;
    color: #555;
    font-size: 1rem;
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
    padding: 24px;
    border-radius: 0 8px 12px 12px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
  }

  .log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid #e1e5eb;
  }

  .year-filter {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .year-filter label {
    font-weight: 600;
    color: #555;
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
    padding: 8px 14px;
    border-radius: 6px;
    font-size: 0.9rem;
  }

  form {
    display: flex;
    flex-direction: column;
  }

  .form-row {
    display: flex;
    gap: 16px;
    margin-bottom: 16px;
  }

  .form-group {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .form-group.small {
    flex: 0 0 100px;
  }

  label {
    font-size: 0.875rem;
    font-weight: 600;
    color: #555;
    margin-bottom: 6px;
  }

  input, textarea {
    padding: 10px 12px;
    border: 1px solid #ddd;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s, box-shadow 0.2s;
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
    margin-top: 16px;
  }

  button {
    padding: 10px 20px;
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

  .btn-icon {
    padding: 6px 10px;
    background: transparent;
    font-size: 1rem;
  }

  .btn-icon:hover {
    background: #f0f0f0;
  }

  .empty-state {
    text-align: center;
    padding: 48px 24px;
    color: #888;
    font-size: 1rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    text-align: left;
    padding: 12px 14px;
    background: #f8f9fb;
    font-size: 0.875rem;
    font-weight: 600;
    color: #555;
    border-bottom: 1px solid #e1e5eb;
  }

  td {
    padding: 12px 14px;
    border-bottom: 1px solid #f0f0f0;
  }

  tr:last-child td {
    border-bottom: none;
  }

  tr:hover {
    background: #fafbfc;
  }

  .date {
    white-space: nowrap;
    color: #666;
  }

  .place {
    font-weight: 500;
  }

  .hours {
    font-weight: 600;
    color: #4a6cf7;
  }

  .notes {
    color: #666;
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    white-space: nowrap;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid #e1e5eb;
  }

  .btn-page {
    padding: 8px 16px;
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
</style>
