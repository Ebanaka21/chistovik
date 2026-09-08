import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Music, Upload, BarChart3, Settings, DollarSign, Users, Eye, TrendingUp, Plus, Edit, Trash2, ExternalLink, LogOut, LayoutDashboard, FileAudio, FileVideo } from 'lucide-react';

type Tab = 'overview' | 'content' | 'pricing' | 'stats' | 'payouts' | 'settings';

export default function AuthorDashboard() {
  const [activeTab, setActiveTab] = useState<Tab>('overview');
  const [showUploadModal, setShowUploadModal] = useState(false);

  const stats = {
    subscribers: 1247,
    revenue: 287450,
    conversion: 12.4,
    views: 45890,
  };

  const content = [
    { id: 1, title: 'Ночной город', type: 'audio', format: 'FLAC', plays: 3420, status: 'published' },
    { id: 2, title: 'Рассвет', type: 'audio', format: 'WAV', plays: 2180, status: 'published' },
    { id: 3, title: 'Making of: Сигналы', type: 'video', format: 'MP4', plays: 1540, status: 'published' },
    { id: 4, title: 'Пульс', type: 'audio', format: 'FLAC', plays: 980, status: 'draft' },
    { id: 5, title: 'Live Session', type: 'video', format: 'MOV', plays: 0, status: 'draft' },
  ];

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: 'overview', label: 'Обзор', icon: <LayoutDashboard className="w-4 h-4" /> },
    { id: 'content', label: 'Контент', icon: <FileAudio className="w-4 h-4" /> },
    { id: 'pricing', label: 'Тарифы', icon: <DollarSign className="w-4 h-4" /> },
    { id: 'stats', label: 'Статистика', icon: <BarChart3 className="w-4 h-4" /> },
    { id: 'payouts', label: 'Выплаты', icon: <DollarSign className="w-4 h-4" /> },
    { id: 'settings', label: 'Настройки', icon: <Settings className="w-4 h-4" /> },
  ];

  return (
    <div className="min-h-screen bg-dark flex">
      {/* Sidebar */}
      <aside className="w-64 glass-card border-r border-dark-border p-4 flex flex-col fixed h-full">
        <Link to="/" className="flex items-center gap-2 mb-8">
          <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
            <Music className="w-5 h-5 text-white" />
          </div>
          <span className="text-lg font-bold gradient-text">Чистовик</span>
        </Link>

        <nav className="flex-1 space-y-1">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition ${activeTab === tab.id ? 'bg-primary/20 text-primary-light' : 'text-gray-400 hover:text-white hover:bg-dark-card'}`}
            >
              {tab.icon}
              {tab.label}
            </button>
          ))}
        </nav>

        <div className="border-t border-dark-border pt-4 space-y-1">
          <Link to="/aleksei-morozov" className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-dark-card transition">
            <ExternalLink className="w-4 h-4" />
            Моя витрина
          </Link>
          <Link to="/" className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-dark-card transition">
            <LogOut className="w-4 h-4" />
            Выйти
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 ml-64 p-8">
        {activeTab === 'overview' && (
          <div>
            <div className="flex items-center justify-between mb-8">
              <div>
                <h1 className="text-3xl font-bold">Добро пожаловать, Алексей!</h1>
                <p className="text-gray-400 mt-1">Вот что происходит на вашей витрине</p>
              </div>
              <button onClick={() => setShowUploadModal(true)} className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-primary hover:bg-primary-dark text-white transition">
                <Plus className="w-4 h-4" />
                Загрузить контент
              </button>
            </div>

            {/* Stats Cards */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <Users className="w-5 h-5 text-primary-light" />
                  <span className="text-xs text-green-400 flex items-center gap-1"><TrendingUp className="w-3 h-3" /> +12%</span>
                </div>
                <div className="text-2xl font-bold">{stats.subscribers}</div>
                <div className="text-sm text-gray-400">Подписчиков</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <DollarSign className="w-5 h-5 text-green-400" />
                  <span className="text-xs text-green-400 flex items-center gap-1"><TrendingUp className="w-3 h-3" /> +8%</span>
                </div>
                <div className="text-2xl font-bold">{stats.revenue.toLocaleString()} ₽</div>
                <div className="text-sm text-gray-400">Доход (месяц)</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <TrendingUp className="w-5 h-5 text-accent" />
                  <span className="text-xs text-green-400 flex items-center gap-1"><TrendingUp className="w-3 h-3" /> +2.1%</span>
                </div>
                <div className="text-2xl font-bold">{stats.conversion}%</div>
                <div className="text-sm text-gray-400">Конверсия</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <Eye className="w-5 h-5 text-blue-400" />
                  <span className="text-xs text-green-400 flex items-center gap-1"><TrendingUp className="w-3 h-3" /> +15%</span>
                </div>
                <div className="text-2xl font-bold">{stats.views.toLocaleString()}</div>
                <div className="text-sm text-gray-400">Просмотров</div>
              </div>
            </div>

            {/* Recent Activity */}
            <div className="glass-card rounded-xl p-6">
              <h3 className="text-lg font-bold mb-4">Последняя активность</h3>
              <div className="space-y-3">
                {[
                  { text: 'Новый подписчик: music_fan_42', time: '2 мин назад', icon: '👤' },
                  { text: 'Продлена подписка: audio_lover', time: '15 мин назад', icon: '🔄' },
                  { text: 'Прослушан трек "Ночной город"', time: '1 час назад', icon: '🎵' },
                  { text: 'Новый подписчик на модуль Видео', time: '3 часа назад', icon: '🎬' },
                  { text: 'Поступление: 1 247 ₽ (комиссия 5%)', time: '5 часов назад', icon: '💰' },
                ].map((item, i) => (
                  <div key={i} className="flex items-center gap-3 py-2 border-b border-dark-border last:border-0">
                    <span className="text-lg">{item.icon}</span>
                    <span className="flex-1 text-sm">{item.text}</span>
                    <span className="text-xs text-gray-500">{item.time}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'content' && (
          <div>
            <div className="flex items-center justify-between mb-8">
              <h1 className="text-3xl font-bold">Управление контентом</h1>
              <button onClick={() => setShowUploadModal(true)} className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-primary hover:bg-primary-dark text-white transition">
                <Upload className="w-4 h-4" />
                Загрузить
              </button>
            </div>

            <div className="glass-card rounded-xl overflow-hidden">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-dark-border">
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Название</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Тип</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Формат</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Прослушиваний</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Статус</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Действия</th>
                  </tr>
                </thead>
                <tbody>
                  {content.map(item => (
                    <tr key={item.id} className="border-b border-dark-border hover:bg-dark-card/50">
                      <td className="px-6 py-4 font-medium">{item.title}</td>
                      <td className="px-6 py-4">
                        <span className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs ${item.type === 'audio' ? 'bg-primary/20 text-primary-light' : 'bg-accent/20 text-accent'}`}>
                          {item.type === 'audio' ? <FileAudio className="w-3 h-3" /> : <FileVideo className="w-3 h-3" />}
                          {item.type === 'audio' ? 'Аудио' : 'Видео'}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-gray-400">{item.format}</td>
                      <td className="px-6 py-4 text-gray-400">{item.plays.toLocaleString()}</td>
                      <td className="px-6 py-4">
                        <span className={`px-2 py-1 rounded text-xs ${item.status === 'published' ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'}`}>
                          {item.status === 'published' ? 'Опубликован' : 'Черновик'}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-2">
                          <button className="p-1.5 rounded hover:bg-dark-card text-gray-400 hover:text-white transition">
                            <Edit className="w-4 h-4" />
                          </button>
                          <button className="p-1.5 rounded hover:bg-dark-card text-gray-400 hover:text-red-400 transition">
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'pricing' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Управление тарифами</h1>
            <div className="grid md:grid-cols-3 gap-6">
              {[
                { id: 'music', title: 'Музыка', enabled: true, price: 199, subscribers: 842 },
                { id: 'video', title: 'Видео', enabled: true, price: 249, subscribers: 405 },
                { id: 'combo', title: 'Комбо', enabled: true, price: 349, subscribers: 312 },
              ].map(plan => (
                <div key={plan.id} className="glass-card rounded-xl p-6">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-bold">{plan.title}</h3>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" defaultChecked className="sr-only peer" />
                      <div className="w-9 h-5 bg-dark-border peer-checked:bg-primary rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all"></div>
                    </label>
                  </div>
                  <div className="mb-4">
                    <label className="text-sm text-gray-400 mb-1 block">Цена (₽/мес)</label>
                    <input
                      type="number"
                      defaultValue={plan.price}
                      className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white"
                    />
                  </div>
                  <div className="text-sm text-gray-400">
                    <span className="text-white font-medium">{plan.subscribers}</span> подписчиков
                  </div>
                  <button className="w-full mt-4 py-2 rounded-lg border border-dark-border hover:border-primary-light text-sm transition">
                    Настроить периоды
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'stats' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Статистика</h1>
            <div className="grid md:grid-cols-2 gap-6">
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Подписчики по модулям</h3>
                <div className="space-y-4">
                  {[
                    { label: 'Музыка', count: 842, color: 'bg-primary', percent: 67 },
                    { label: 'Видео', count: 405, color: 'bg-accent', percent: 32 },
                    { label: 'Комбо', count: 312, color: 'bg-green-500', percent: 25 },
                  ].map(item => (
                    <div key={item.label}>
                      <div className="flex justify-between text-sm mb-1">
                        <span>{item.label}</span>
                        <span className="text-gray-400">{item.count}</span>
                      </div>
                      <div className="h-2 bg-dark rounded-full overflow-hidden">
                        <div className={`h-full ${item.color} rounded-full`} style={{ width: `${item.percent}%` }} />
                      </div>
                    </div>
                  ))}
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Доход по месяцам</h3>
                <div className="flex items-end gap-2 h-40">
                  {[40, 55, 45, 60, 75, 65, 80, 90, 85, 95, 88, 100].map((h, i) => (
                    <div key={i} className="flex-1 flex flex-col items-center gap-1">
                      <div className="w-full bg-primary/60 rounded-t" style={{ height: `${h}%` }} />
                      <span className="text-xs text-gray-500">{i + 1}</span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Конверсия тизер → подписка</h3>
                <div className="text-center py-8">
                  <div className="text-5xl font-bold gradient-text">12.4%</div>
                  <p className="text-gray-400 mt-2">Средняя конверсия за последние 30 дней</p>
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Топ треки</h3>
                <div className="space-y-3">
                  {[
                    { title: 'Ночной город', plays: 3420 },
                    { title: 'Рассвет', plays: 2180 },
                    { title: 'Пульс', plays: 980 },
                    { title: 'Тишина', plays: 756 },
                    { title: 'Эхо', plays: 623 },
                  ].map((track, i) => (
                    <div key={i} className="flex items-center gap-3">
                      <span className="text-gray-500 text-sm w-5">{i + 1}</span>
                      <span className="flex-1 text-sm">{track.title}</span>
                      <span className="text-sm text-gray-400">{track.plays.toLocaleString()}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'payouts' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Выплаты</h1>
            <div className="grid md:grid-cols-3 gap-4 mb-8">
              <div className="glass-card rounded-xl p-5">
                <div className="text-sm text-gray-400 mb-1">Доступно к выводу</div>
                <div className="text-2xl font-bold text-green-400">142 350 ₽</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="text-sm text-gray-400 mb-1">В обработке</div>
                <div className="text-2xl font-bold text-accent">25 000 ₽</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="text-sm text-gray-400 mb-1">Всего выплачено</div>
                <div className="text-2xl font-bold">1 245 800 ₽</div>
              </div>
            </div>
            <div className="glass-card rounded-xl p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-bold">История выплат</h3>
                <button className="px-4 py-2 rounded-lg bg-primary hover:bg-primary-dark text-white text-sm transition">
                  Запросить выплату
                </button>
              </div>
              <div className="space-y-3">
                {[
                  { date: '15.01.2026', amount: 45000, status: 'Выплачено', method: 'СБП' },
                  { date: '01.01.2026', amount: 38500, status: 'Выплачено', method: 'Карта МИР' },
                  { date: '15.12.2025', amount: 52000, status: 'Выплачено', method: 'СБП' },
                  { date: '01.12.2025', amount: 41200, status: 'Выплачено', method: 'Р/счёт' },
                ].map((p, i) => (
                  <div key={i} className="flex items-center justify-between py-3 border-b border-dark-border last:border-0">
                    <div>
                      <div className="font-medium">{p.amount.toLocaleString()} ₽</div>
                      <div className="text-sm text-gray-400">{p.date} • {p.method}</div>
                    </div>
                    <span className="px-2 py-1 rounded text-xs bg-green-500/20 text-green-400">{p.status}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Настройки витрины</h1>
            <div className="space-y-6 max-w-2xl">
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Основная информация</h3>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Имя автора</label>
                    <input defaultValue="Алексей Морозов" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Описание</label>
                    <textarea defaultValue="Электронный музыкант и продюсер." rows={3} className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white resize-none" />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">URL витрины</label>
                    <div className="flex items-center gap-2">
                      <span className="text-gray-400 text-sm">chistovik.ru/</span>
                      <input defaultValue="aleksei-morozov" className="flex-1 px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                    </div>
                  </div>
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Социальные сети</h3>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Telegram</label>
                    <input defaultValue="@morozov_music" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">VK</label>
                    <input defaultValue="vk.com/morozov_music" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">YouTube</label>
                    <input defaultValue="youtube.com/@AlekseiMorozov" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                </div>
              </div>
              <button className="px-6 py-3 rounded-lg bg-primary hover:bg-primary-dark text-white font-medium transition">
                Сохранить изменения
              </button>
            </div>
          </div>
        )}
      </main>

      {/* Upload Modal */}
      {showUploadModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm" onClick={() => setShowUploadModal(false)}>
          <div className="glass-card rounded-2xl p-8 max-w-lg w-full" onClick={e => e.stopPropagation()}>
            <h3 className="text-2xl font-bold mb-6">Загрузить контент</h3>
            <div className="space-y-4">
              <div>
                <label className="text-sm text-gray-400 mb-1 block">Тип контента</label>
                <div className="flex gap-2">
                  <button className="flex-1 py-2.5 rounded-lg bg-primary/20 text-primary-light border border-primary-light text-sm font-medium">
                    🎵 Музыка
                  </button>
                  <button className="flex-1 py-2.5 rounded-lg glass-card text-gray-300 text-sm font-medium">
                    🎬 Видео
                  </button>
                </div>
              </div>
              <div>
                <label className="text-sm text-gray-400 mb-1 block">Название</label>
                <input placeholder="Название трека / видео" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
              </div>
              <div>
                <label className="text-sm text-gray-400 mb-1 block">Описание</label>
                <textarea placeholder="Описание контента" rows={2} className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white resize-none" />
              </div>
              <div>
                <label className="text-sm text-gray-400 mb-2 block">Файл</label>
                <div className="border-2 border-dashed border-dark-border rounded-xl p-8 text-center hover:border-primary-light transition cursor-pointer">
                  <Upload className="w-8 h-8 text-gray-400 mx-auto mb-2" />
                  <p className="text-sm text-gray-400">Перетащите файл или нажмите для выбора</p>
                  <p className="text-xs text-gray-500 mt-1">FLAC, WAV, MP3, MP4, MOV • до 2 ГБ</p>
                </div>
              </div>
              <div>
                <label className="text-sm text-gray-400 mb-1 block">Тизер (секунды)</label>
                <div className="flex gap-2">
                  <input type="number" defaultValue={0} className="w-20 px-3 py-2 rounded-lg bg-dark border border-dark-border text-white text-center" />
                  <span className="text-gray-400 self-center">—</span>
                  <input type="number" defaultValue={15} className="w-20 px-3 py-2 rounded-lg bg-dark border border-dark-border text-white text-center" />
                  <span className="text-sm text-gray-400 self-center">сек (макс 15 для аудио / 60 для видео)</span>
                </div>
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button onClick={() => setShowUploadModal(false)} className="flex-1 py-3 rounded-lg glass-card text-gray-300 font-medium transition hover:border-primary-light">
                Отмена
              </button>
              <button onClick={() => setShowUploadModal(false)} className="flex-1 py-3 rounded-lg bg-primary hover:bg-primary-dark text-white font-medium transition">
                Загрузить
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
