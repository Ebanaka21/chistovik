import { useState, type ReactNode } from 'react';
import { Link } from 'react-router-dom';
import { Music, Play, Pause, Heart, CreditCard, BookOpen, User, Settings, LogOut, Bell, Shield } from 'lucide-react';

type Tab = 'library' | 'subscriptions' | 'payments' | 'settings';

export default function UserDashboard() {
  const [activeTab, setActiveTab] = useState<Tab>('library');
  const [playingId, setPlayingId] = useState<string | null>(null);

  const subscriptions = [
    { id: '1', author: 'Алексей Морозов', avatar: '🎹', modules: ['Музыка', 'Видео'], price: 349, nextPayment: '15.02.2026', status: 'active' },
    { id: '2', author: 'Мария Светлова', avatar: '🎨', modules: ['Видео'], price: 249, nextPayment: '22.02.2026', status: 'active' },
    { id: '3', author: 'Дмитрий Волков', avatar: '🎸', modules: ['Музыка'], price: 199, nextPayment: '01.03.2026', status: 'active' },
  ];

  const library = [
    { id: '1', title: 'Ночной город', author: 'Алексей Морозов', type: 'audio', format: 'FLAC', duration: '4:23', cover: '🌃' },
    { id: '2', title: 'Рассвет', author: 'Алексей Морозов', type: 'audio', format: 'WAV', duration: '3:45', cover: '🌅' },
    { id: '3', title: 'Making of: Сигналы', author: 'Алексей Морозов', type: 'video', duration: '12:34', cover: '🎬' },
    { id: '4', title: 'Пульс', author: 'Алексей Морозов', type: 'audio', format: 'FLAC', duration: '5:12', cover: '💓' },
    { id: '5', title: 'Digital Art Process', author: 'Мария Светлова', type: 'video', duration: '18:45', cover: '🖌️' },
    { id: '6', title: 'Новый альбом (demo)', author: 'Дмитрий Волков', type: 'audio', format: 'FLAC', duration: '4:01', cover: '🎸' },
    { id: '7', title: 'Live Session', author: 'Алексей Морозов', type: 'video', duration: '45:20', cover: '🎤' },
    { id: '8', title: 'Тишина', author: 'Алексей Морозов', type: 'audio', format: 'FLAC', duration: '6:01', cover: '🤫' },
  ];

  const payments = [
    { id: '1', date: '15.01.2026', author: 'Алексей Морозов', amount: 349, method: 'Карта МИР ••4523', status: 'Оплачено' },
    { id: '2', date: '10.01.2026', author: 'Мария Светлова', amount: 249, method: 'СБП', status: 'Оплачено' },
    { id: '3', date: '05.01.2026', author: 'Дмитрий Волков', amount: 199, method: 'Карта МИР ••4523', status: 'Оплачено' },
    { id: '4', date: '15.12.2025', author: 'Алексей Морозов', amount: 349, method: 'Карта МИР ••4523', status: 'Оплачено' },
    { id: '5', date: '10.12.2025', author: 'Мария Светлова', amount: 249, method: 'СБП', status: 'Оплачено' },
  ];

  const tabs: { id: Tab; label: string; icon: ReactNode }[] = [
    { id: 'library', label: 'Моя библиотека', icon: <BookOpen className="w-4 h-4" /> },
    { id: 'subscriptions', label: 'Подписки', icon: <Heart className="w-4 h-4" /> },
    { id: 'payments', label: 'Платежи', icon: <CreditCard className="w-4 h-4" /> },
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
          <button className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-dark-card transition">
            <Bell className="w-4 h-4" />
            Уведомления
            <span className="ml-auto w-5 h-5 rounded-full bg-primary text-white text-xs flex items-center justify-center">3</span>
          </button>
          <Link to="/" className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-dark-card transition">
            <LogOut className="w-4 h-4" />
            Выйти
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 ml-64 p-8">
        {activeTab === 'library' && (
          <div>
            <div className="flex items-center justify-between mb-8">
              <div>
                <h1 className="text-3xl font-bold">Моя библиотека</h1>
                <p className="text-gray-400 mt-1">Весь контент из ваших подписок</p>
              </div>
              <div className="flex gap-2">
                <button className="px-4 py-2 rounded-lg bg-primary/20 text-primary-light text-sm font-medium">Все</button>
                <button className="px-4 py-2 rounded-lg glass-card text-gray-300 text-sm font-medium">🎵 Музыка</button>
                <button className="px-4 py-2 rounded-lg glass-card text-gray-300 text-sm font-medium">🎬 Видео</button>
              </div>
            </div>

            <div className="space-y-2">
              {library.map(item => (
                <div key={item.id} className="glass-card rounded-xl p-4 hover:border-primary-light/50 transition group">
                  <div className="flex items-center gap-4">
                    <div className="w-12 h-12 rounded-lg bg-dark flex items-center justify-center text-xl flex-shrink-0">
                      {item.cover}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="font-medium truncate">{item.title}</h3>
                        {item.type === 'audio' && (
                          <span className="px-2 py-0.5 rounded text-xs bg-primary/20 text-primary-light">{item.format}</span>
                        )}
                      </div>
                      <div className="text-sm text-gray-400">{item.author} • {item.duration}</div>
                    </div>
                    <button
                      onClick={() => setPlayingId(playingId === item.id ? null : item.id)}
                      className="w-10 h-10 rounded-full bg-primary/20 hover:bg-primary/40 flex items-center justify-center transition opacity-0 group-hover:opacity-100"
                    >
                      {playingId === item.id ? (
                        <Pause className="w-4 h-4 text-primary-light" />
                      ) : (
                        <Play className="w-4 h-4 text-primary-light ml-0.5" />
                      )}
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'subscriptions' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Мои подписки</h1>
            <div className="space-y-4">
              {subscriptions.map(sub => (
                <div key={sub.id} className="glass-card rounded-xl p-6">
                  <div className="flex items-center gap-4">
                    <div className="w-14 h-14 rounded-xl bg-dark flex items-center justify-center text-3xl">
                      {sub.avatar}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className="text-lg font-bold">{sub.author}</h3>
                        <span className="px-2 py-0.5 rounded text-xs bg-green-500/20 text-green-400">Активна</span>
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        {sub.modules.map(m => (
                          <span key={m} className="px-2 py-0.5 rounded text-xs bg-primary/20 text-primary-light">{m}</span>
                        ))}
                      </div>
                      <div className="text-sm text-gray-400 mt-1">
                        Следующее списание: {sub.nextPayment} • {sub.price} ₽/мес
                      </div>
                    </div>
                    <div className="flex gap-2">
                      <Link to={`/${sub.author.toLowerCase().replace(' ', '-')}`} className="px-4 py-2 rounded-lg glass-card text-sm text-gray-300 hover:border-primary-light transition">
                        Витрина
                      </Link>
                      <button className="px-4 py-2 rounded-lg border border-red-500/30 text-red-400 text-sm hover:bg-red-500/10 transition">
                        Отменить
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            <div className="glass-card rounded-xl p-6 mt-6">
              <h3 className="text-lg font-bold mb-2">Общие расходы</h3>
              <div className="text-3xl font-bold gradient-text">797 ₽/мес</div>
              <p className="text-sm text-gray-400 mt-1">Сумма активных подписок</p>
            </div>
          </div>
        )}

        {activeTab === 'payments' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">История платежей</h1>
            <div className="glass-card rounded-xl overflow-hidden">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-dark-border">
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Дата</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Автор</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Сумма</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Способ</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Статус</th>
                  </tr>
                </thead>
                <tbody>
                  {payments.map(p => (
                    <tr key={p.id} className="border-b border-dark-border hover:bg-dark-card/50">
                      <td className="px-6 py-4 text-gray-400">{p.date}</td>
                      <td className="px-6 py-4 font-medium">{p.author}</td>
                      <td className="px-6 py-4">{p.amount} ₽</td>
                      <td className="px-6 py-4 text-gray-400 text-sm">{p.method}</td>
                      <td className="px-6 py-4">
                        <span className="px-2 py-1 rounded text-xs bg-green-500/20 text-green-400">{p.status}</span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <div className="glass-card rounded-xl p-6 mt-6">
              <h3 className="text-lg font-bold mb-4">Способы оплаты</h3>
              <div className="space-y-3">
                <div className="flex items-center gap-3 p-3 rounded-lg bg-dark border border-primary-light">
                  <CreditCard className="w-5 h-5 text-primary-light" />
                  <div className="flex-1">
                    <div className="font-medium text-sm">Карта МИР ••4523</div>
                    <div className="text-xs text-gray-400">Основная • до 12/28</div>
                  </div>
                  <span className="px-2 py-0.5 rounded text-xs bg-primary/20 text-primary-light">Основная</span>
                </div>
                <button className="w-full py-3 rounded-lg border border-dashed border-dark-border text-gray-400 hover:text-white hover:border-primary-light transition text-sm">
                  + Добавить карту
                </button>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Настройки аккаунта</h1>
            <div className="space-y-6 max-w-2xl">
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Профиль</h3>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Имя</label>
                    <input defaultValue="Иван Петров" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Email</label>
                    <input defaultValue="ivan@example.com" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Телефон</label>
                    <input defaultValue="+7 (999) 123-45-67" className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Уведомления</h3>
                <div className="space-y-3">
                  {[
                    { label: 'Email-уведомления о списаниях', enabled: true },
                    { label: 'Telegram-уведомления', enabled: true },
                    { label: 'Новый контент от авторов', enabled: false },
                    { label: 'Напоминание за 3 дня до списания', enabled: true },
                  ].map((item, i) => (
                    <div key={i} className="flex items-center justify-between">
                      <span className="text-sm">{item.label}</span>
                      <label className="relative inline-flex items-center cursor-pointer">
                        <input type="checkbox" defaultChecked={item.enabled} className="sr-only peer" />
                        <div className="w-9 h-5 bg-dark-border peer-checked:bg-primary rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-0.5 after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all"></div>
                      </label>
                    </div>
                  ))}
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Безопасность</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium text-sm">Смена пароля</div>
                      <div className="text-xs text-gray-400">Последняя смена: 3 месяца назад</div>
                    </div>
                    <button className="px-4 py-2 rounded-lg glass-card text-sm hover:border-primary-light transition">Изменить</button>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium text-sm">Активные сессии</div>
                      <div className="text-xs text-gray-400">2 устройства (макс. 2)</div>
                    </div>
                    <button className="px-4 py-2 rounded-lg glass-card text-sm hover:border-primary-light transition">Управление</button>
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
    </div>
  );
}
