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

  onMount(async () => {
    entries = await invoke('get_entries');
  });

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
  }

  async function deleteEntry(id: string) {
    entries = await invoke('delete_entry', { id });
  }

  function getTotalHours(): number {
    return entries.reduce((sum, e) => sum + e.hours, 0);
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr + 'T00:00:00').toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }
</script>

<main>
  <header>
    <h1>📋 Volunteering Log</h1>
    <div class="total-hours">
      Total: <strong>{getTotalHours().toFixed(1)}</strong> hours
    </div>
  </header>

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
        rows="2"
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

  <section class="entries">
    {#if entries.length === 0}
      <p class="empty-state">No volunteer hours logged yet. Add your first entry above!</p>
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
          {#each entries.sort((a, b) => b.date.localeCompare(a.date)) as entry}
            <tr class:editing={editingId === entry.id}>
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
    {/if}
  </section>
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
    margin-bottom: 24px;
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

  form {
    background: white;
    padding: 20px;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
    margin-bottom: 24px;
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

  .entries {
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.08);
    overflow: hidden;
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
    padding: 14px 16px;
    background: #f8f9fb;
    font-size: 0.875rem;
    font-weight: 600;
    color: #555;
    border-bottom: 1px solid #e1e5eb;
  }

  td {
    padding: 14px 16px;
    border-bottom: 1px solid #f0f0f0;
  }

  tr:last-child td {
    border-bottom: none;
  }

  tr:hover {
    background: #fafbfc;
  }

  tr.editing {
    background: #f0f4ff;
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
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    white-space: nowrap;
  }
</style>
