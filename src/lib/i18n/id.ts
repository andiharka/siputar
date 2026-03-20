export interface DayNames { 1: string; 2: string; 3: string; 4: string; 5: string; 6: string; 7: string; fullName: { 1: string; 2: string; 3: string; 4: string; 5: string; 6: string; 7: string }; }

export interface Translations {
  app: { name: string };
  nav: { schedules: string; audio: string };
  schedule: { title: string; addSchedule: string; noSchedules: string; time: string; activeDays: string; enabled: string; statusEnabled: string; statusDisabled: string; totalDuration: string; notifications: string; addNotification: string; minutesBefore: string; deleteConfirm: string };
  media: { title: string; addMedia: string; loopCount: string; volume: string; fileMissing: string; noMedia: string; deleteConfirm: string; loopForever: string };
  settings: { title: string; theme: string; themeLight: string; themeDark: string; themeAuto: string; language: string; runOnStartup: string; apiKey: string; apiKeyPlaceholder: string; apiKeyHint: string; testConnection: string; testSuccess: string; testFailed: string; audioFolder: string; audioFolderHint: string; browse: string };
  actions: { save: string; revert: string; close: string; edit: string; delete: string; add: string; play: string; cancel: string; confirm: string; moveUp: string; moveDown: string; generate: string; download: string; openFolder: string; stop: string };
  tts: { title: string; generateAudio: string; credits: string; creditsRemaining: string; connectionOnline: string; connectionOffline: string; text: string; textPlaceholder: string; charLimit: string; voice: string; voicePlaceholder: string; model: string; modelPlaceholder: string; language: string; speed: string; speedHint: string; stability: string; stabilityHint: string; similarity: string; similarityHint: string; generating: string; completed: string; failed: string; noHistory: string; deleteConfirm: string; downloadSuccess: string; downloadFailed: string; offline: string; sync: string; syncing: string };
  days: DayNames;
  unsaved: { title: string; message: string; save: string; discard: string; cancel: string };
  playback: { nowPlaying: string; paused: string; pause: string; resume: string; stop: string };
  status: { active: string; paused: string };
}

export const id: Translations = {
  app: { name: 'Disperpusip Bersuara' },
  nav: { schedules: 'Daftar Jadwal', audio: 'Buat Audio' },
  schedule: {
    title: 'Jadwal',
    addSchedule: 'Tambah Jadwal',
    noSchedules: 'Belum ada jadwal. Klik tombol + untuk menambahkan.',
    time: 'Waktu',
    activeDays: 'Hari Aktif',
    enabled: 'Aktif',
    statusEnabled: 'Pemutar berjalan sesuai jadwal (klik untuk menjeda)',
    statusDisabled: 'Pemutar tidak akan berjalan (klik untuk menyalakan)',
    totalDuration: 'Total Durasi',
    notifications: 'Notifikasi',
    addNotification: 'Tambah Notifikasi',
    minutesBefore: 'menit sebelumnya',
    deleteConfirm: 'Hapus jadwal ini beserta semua medianya?',
  },
  media: {
    title: 'Media',
    addMedia: 'Tambah Media',
    loopCount: 'Jumlah Pengulangan',
    volume: 'Volume',
    fileMissing: 'File tidak ditemukan',
    noMedia: 'Belum ada media',
    deleteConfirm: 'Hapus media ini dari jadwal?',
    loopForever: 'Loop terus',
  },
  settings: {
    title: 'Pengaturan',
    theme: 'Tema',
    themeLight: 'Terang',
    themeDark: 'Gelap',
    themeAuto: 'Otomatis',
    language: 'Bahasa',
    runOnStartup: 'Jalankan saat komputer menyala',
    apiKey: 'API Key ElevenLabs',
    apiKeyPlaceholder: 'Masukkan API key',
    apiKeyHint: 'Dapatkan API key dari elevenlabs.io/app/settings/api-keys',
    testConnection: 'Tes Koneksi',
    testSuccess: 'Koneksi berhasil!',
    testFailed: 'Koneksi gagal',
    audioFolder: 'Folder Audio TTS',
    audioFolderHint: 'Lokasi penyimpanan file audio yang dihasilkan',
    browse: 'Pilih Folder',
  },
  actions: {
    save: 'Simpan',
    revert: 'Batalkan Perubahan',
    close: 'Tutup',
    edit: 'Edit',
    delete: 'Hapus',
    play: 'Putar Sekarang',
    add: 'Tambah',
    cancel: 'Batal',
    confirm: 'Konfirmasi',
    moveUp: 'Pindah ke atas',
    moveDown: 'Pindah ke bawah',
    generate: 'Buat Audio',
    download: 'Unduh',
    openFolder: 'Buka Folder',
    stop: 'Berhenti',
  },
  tts: {
    title: 'Text-to-Speech',
    generateAudio: 'Buat Audio Baru',
    credits: 'Kredit',
    creditsRemaining: 'karakter tersisa',
    connectionOnline: 'Mode online - dapat generate audio',
    connectionOffline: 'Mode offline - hanya dapat memutar file lokal',
    text: 'Teks',
    textPlaceholder: 'Masukkan teks yang ingin diubah menjadi audio...',
    charLimit: 'karakter',
    voice: 'Suara',
    voicePlaceholder: 'Pilih suara',
    model: 'Model',
    modelPlaceholder: 'Pilih model',
    language: 'Bahasa',
    speed: 'Kecepatan',
    speedHint: 'Kecepatan bicara (0.25 - 4.0)',
    stability: 'Stabilitas',
    stabilityHint: 'Nilai tinggi = lebih konsisten, nilai rendah = lebih ekspresif',
    similarity: 'Kesamaan Suara',
    similarityHint: 'Seberapa mirip dengan suara asli',
    generating: 'Sedang membuat...',
    completed: 'Selesai',
    failed: 'Gagal',
    noHistory: 'Belum ada riwayat audio',
    deleteConfirm: 'Hapus audio ini?',
    downloadSuccess: 'Audio berhasil diunduh',
    downloadFailed: 'Gagal mengunduh audio',
    offline: 'Mode offline - hanya dapat memutar file lokal',
    sync: 'Sinkronkan',
    syncing: 'Menyinkronkan...',
  },
  days: {
    1: 'Sen', 2: 'Sel', 3: 'Rab', 4: 'Kam', 5: 'Jum', 6: 'Sab', 7: 'Min',
    fullName: { 1: 'Senin', 2: 'Selasa', 3: 'Rabu', 4: 'Kamis', 5: 'Jumat', 6: 'Sabtu', 7: 'Minggu' },
  },
  unsaved: {
    title: 'Perubahan belum disimpan',
    message: 'Apakah Anda ingin menyimpan perubahan sebelum keluar?',
    save: 'Simpan & Tutup',
    discard: 'Buang Perubahan',
    cancel: 'Batal',
  },
  playback: {
    nowPlaying: 'Sedang diputar',
    paused: 'Dijeda',
    pause: 'Jeda',
    resume: 'Lanjutkan',
    stop: 'Hentikan',
  },
  status: {
    active: 'Aktif',
    paused: 'Dijeda',
  },
};
