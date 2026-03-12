export interface DayNames { 1: string; 2: string; 3: string; 4: string; 5: string; 6: string; 7: string; fullName: { 1: string; 2: string; 3: string; 4: string; 5: string; 6: string; 7: string }; }

export interface Translations {
  app: { name: string };
  schedule: { title: string; addSchedule: string; noSchedules: string; time: string; activeDays: string; enabled: string; totalDuration: string; notifications: string; addNotification: string; minutesBefore: string; deleteConfirm: string };
  media: { title: string; addMedia: string; loopCount: string; volume: string; fileMissing: string; noMedia: string; deleteConfirm: string; loopForever: string };
  settings: { title: string; theme: string; themeLight: string; themeDark: string; themeAuto: string; language: string; runOnStartup: string };
  actions: { save: string; revert: string; close: string; delete: string; add: string; play: string; cancel: string; confirm: string; moveUp: string; moveDown: string };
  days: DayNames;
  unsaved: { title: string; message: string; save: string; discard: string; cancel: string };
  playback: { nowPlaying: string; paused: string; pause: string; resume: string; stop: string };
  status: { active: string; paused: string };
}

export const id: Translations = {
  app: { name: 'Disperpusip Bersuara' },
  schedule: {
    title: 'Jadwal',
    addSchedule: 'Tambah Jadwal',
    noSchedules: 'Belum ada jadwal. Klik tombol + untuk menambahkan.',
    time: 'Waktu',
    activeDays: 'Hari Aktif',
    enabled: 'Aktif',
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
  },
  actions: {
    save: 'Simpan',
    revert: 'Batalkan Perubahan',
    close: 'Tutup',
    delete: 'Hapus',
    play: 'Putar Sekarang',
    add: 'Tambah',
    cancel: 'Batal',
    confirm: 'Konfirmasi',
    moveUp: 'Pindah ke atas',
    moveDown: 'Pindah ke bawah',
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
