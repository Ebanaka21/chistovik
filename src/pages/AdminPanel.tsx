import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Music, Users, DollarSign, FileText, Shield, AlertTriangle, CheckCircle, XCircle, Eye, Ban, Search } from 'lucide-react';

type Tab = 'overview' | 'users' | 'authors' | 'transactions' | 'moderation' | 'settings';

export default function AdminPanel() {
  const [activeTab, setActiveTab] = useState<Tab>('overview');

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: 'overview', label: 'Обзор', icon: <Shield className="w-4 h-4" /> },
    { id: 'users', label: 'Пользователи', icon: <Users className="w-4 h-4" /> },
    { id: 'authors', label: 'Авторы', icon: <Music className="w-4 h-4" /> },
    { id: 'transactions', label: 'Транзакции', icon: <DollarSign className="w-4 h-4" /> },
    { id: 'moderation', label: 'Модерация', icon: <AlertTriangle className="w-4 h-4" /> },
    { id: 'settings', label: 'Настройки', icon: <FileText className="w-4 h-4" /> },
  ];

  return (
    <div className="min-h-screen bg-dark flex">
      {/* Sidebar */}
      <aside className="w-64 glass-card border-r border-dark-border p-4 flex flex-col fixed h-full">
        <Link to="/" className="flex items-center gap-2 mb-2">
          <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
            <Music className="w-5 h-5 text-white" />
          </div>
          <span className="text-lg font-bold gradient-text">Чистовик</span>
        </Link>
        <div className="px-3 py-1 mb-6">
          <span className="text-xs px-2 py-0.5 rounded bg-red-500/20 text-red-400 font-medium">ADMIN</span>
        </div>

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

        <div className="border-t border-dark-border pt-4">
          <Link to="/" className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-dark-card transition">
            ← Вернуться на сайт
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 ml-64 p-8">
        {activeTab === 'overview' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Панель администратора</h1>
            
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <Users className="w-5 h-5 text-primary-light" />
                  <span className="text-xs text-green-400">+24%</span>
                </div>
                <div className="text-2xl font-bold">12 458</div>
                <div className="text-sm text-gray-400">Пользователей</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <Music className="w-5 h-5 text-accent" />
                  <span className="text-xs text-green-400">+3</span>
                </div>
                <div className="text-2xl font-bold">186</div>
                <div className="text-sm text-gray-400">Авторов</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <DollarSign className="w-5 h-5 text-green-400" />
                  <span className="text-xs text-green-400">+18%</span>
                </div>
                <div className="text-2xl font-bold">2.4M ₽</div>
                <div className="text-sm text-gray-400">MRR платформы</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="flex items-center justify-between mb-3">
                  <DollarSign className="w-5 h-5 text-blue-400" />
                  <span className="text-xs text-green-400">+12%</span>
                </div>
                <div className="text-2xl font-bold">192K ₽</div>
                <div className="text-sm text-gray-400">Комиссия (8%)</div>
              </div>
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Новые авторы (ожидают модерации)</h3>
                <div className="space-y-3">
                  {[
                    { name: 'Елена Тихонова', type: 'Подкасты', date: 'Сегодня' },
                    { name: 'Роман Козлов', type: 'Электронная музыка', date: 'Вчера' },
                    { name: 'Ольга Новикова', type: 'Мастер-классы', date: '2 дня назад' },
                  ].map((a, i) => (
                    <div key={i} className="flex items-center justify-between py-2 border-b border-dark-border last:border-0">
                      <div>
                        <div className="font-medium text-sm">{a.name}</div>
                        <div className="text-xs text-gray-400">{a.type} • {a.date}</div>
                      </div>
                      <div className="flex gap-1">
                        <button className="p-1.5 rounded bg-green-500/20 text-green-400 hover:bg-green-500/30">
                          <CheckCircle className="w-4 h-4" />
                        </button>
                        <button className="p-1.5 rounded bg-red-500/20 text-red-400 hover:bg-red-500/30">
                          <XCircle className="w-4 h-4" />
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Жалобы на контент</h3>
                <div className="space-y-3">
                  {[
                    { content: 'Трек "Тёмная сторона"', author: 'Анонимный', reason: 'Нарушение АП', date: '1 час назад' },
                    { content: 'Видео "Studio Vlog"', author: 'user_234', reason: 'Неприемлемый контент', date: '3 часа назад' },
                  ].map((c, i) => (
                    <div key={i} className="flex items-center justify-between py-2 border-b border-dark-border last:border-0">
                      <div>
                        <div className="font-medium text-sm">{c.content}</div>
                        <div className="text-xs text-gray-400">{c.reason} • {c.date}</div>
                      </div>
                      <button className="p-1.5 rounded glass-card hover:border-primary-light">
                        <Eye className="w-4 h-4 text-gray-400" />
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'users' && (
          <div>
            <div className="flex items-center justify-between mb-8">
              <h1 className="text-3xl font-bold">Пользователи</h1>
              <div className="relative">
                <input placeholder="Поиск..." className="pl-10 pr-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white text-sm w-64" />
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
              </div>
            </div>
            <div className="glass-card rounded-xl overflow-hidden">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-dark-border">
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Пользователь</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Email</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Подписки</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Регистрация</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Статус</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Действия</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    { name: 'Иван Петров', email: 'ivan@mail.ru', subs: 3, date: '12.01.2025', status: 'active' },
                    { name: 'Мария Сидорова', email: 'maria@gmail.com', subs: 1, date: '05.03.2025', status: 'active' },
                    { name: 'Артём Козлов', email: 'artem@yandex.ru', subs: 5, date: '22.06.2025', status: 'active' },
                    { name: 'Наталья Иванова', email: 'natasha@mail.ru', subs: 0, date: '10.11.2025', status: 'suspended' },
                    { name: 'Денис Волков', email: 'denis@inbox.ru', subs: 2, date: '01.12.2025', status: 'active' },
                  ].map((u, i) => (
                    <tr key={i} className="border-b border-dark-border hover:bg-dark-card/50">
                      <td className="px-6 py-4 font-medium">{u.name}</td>
                      <td className="px-6 py-4 text-gray-400 text-sm">{u.email}</td>
                      <td className="px-6 py-4">{u.subs}</td>
                      <td className="px-6 py-4 text-gray-400 text-sm">{u.date}</td>
                      <td className="px-6 py-4">
                        <span className={`px-2 py-1 rounded text-xs ${u.status === 'active' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'}`}>
                          {u.status === 'active' ? 'Активен' : 'Заблокирован'}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <button className="p-1.5 rounded hover:bg-dark-card text-gray-400 hover:text-red-400 transition">
                          <Ban className="w-4 h-4" />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'authors' && (
          <div>
            <div className="flex items-center justify-between mb-8">
              <h1 className="text-3xl font-bold">Авторы</h1>
              <div className="relative">
                <input placeholder="Поиск..." className="pl-10 pr-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white text-sm w-64" />
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
              </div>
            </div>
            <div className="glass-card rounded-xl overflow-hidden">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-dark-border">
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Автор</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Специализация</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Подписчики</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Доход</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Верификация</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Действия</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    { name: 'Алексей Морозов', spec: 'Электронная музыка', subs: 1247, revenue: '287K ₽', verified: true },
                    { name: 'Мария Светлова', spec: 'Видео-арт', subs: 892, revenue: '198K ₽', verified: true },
                    { name: 'Дмитрий Волков', spec: 'Инди-рок', subs: 2103, revenue: '412K ₽', verified: true },
                    { name: 'Анна Козлова', spec: 'Мастер-классы', subs: 567, revenue: '156K ₽', verified: false },
                    { name: 'Елена Тихонова', spec: 'Подкасты', subs: 0, revenue: '0 ₽', verified: false },
                  ].map((a, i) => (
                    <tr key={i} className="border-b border-dark-border hover:bg-dark-card/50">
                      <td className="px-6 py-4 font-medium">{a.name}</td>
                      <td className="px-6 py-4 text-gray-400 text-sm">{a.spec}</td>
                      <td className="px-6 py-4">{a.subs.toLocaleString()}</td>
                      <td className="px-6 py-4">{a.revenue}</td>
                      <td className="px-6 py-4">
                        <span className={`px-2 py-1 rounded text-xs ${a.verified ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'}`}>
                          {a.verified ? '✓ Верифицирован' : '⏳ Ожидает'}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex gap-1">
                          <button className="p-1.5 rounded hover:bg-dark-card text-gray-400 hover:text-white transition">
                            <Eye className="w-4 h-4" />
                          </button>
                          {!a.verified && (
                            <button className="p-1.5 rounded hover:bg-dark-card text-green-400 transition">
                              <CheckCircle className="w-4 h-4" />
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'transactions' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Транзакции</h1>
            <div className="grid grid-cols-3 gap-4 mb-6">
              <div className="glass-card rounded-xl p-5">
                <div className="text-sm text-gray-400 mb-1">За сегодня</div>
                <div className="text-xl font-bold">87 450 ₽</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="text-sm text-gray-400 mb-1">Комиссия платформы</div>
                <div className="text-xl font-bold text-green-400">6 996 ₽</div>
              </div>
              <div className="glass-card rounded-xl p-5">
                <div className="text-sm text-gray-400 mb-1">Возвраты</div>
                <div className="text-xl font-bold text-red-400">1 200 ₽</div>
              </div>
            </div>
            <div className="glass-card rounded-xl overflow-hidden">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-dark-border">
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">ID</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Дата</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Пользователь</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Автор</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Сумма</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Комиссия</th>
                    <th className="text-left px-6 py-4 text-sm font-medium text-gray-400">Статус</th>
                  </tr>
                </thead>
                <tbody>
                  {[
                    { id: 'TX-8842', date: '15.01 14:23', user: 'Иван П.', author: 'А. Морозов', amount: 349, commission: 28, status: 'success' },
                    { id: 'TX-8841', date: '15.01 13:45', user: 'Мария С.', author: 'М. Светлова', amount: 249, commission: 20, status: 'success' },
                    { id: 'TX-8840', date: '15.01 12:10', user: 'Артём К.', author: 'Д. Волков', amount: 199, commission: 16, status: 'success' },
                    { id: 'TX-8839', date: '15.01 11:30', user: 'Денис В.', author: 'А. Морозов', amount: 199, commission: 16, status: 'refund' },
                    { id: 'TX-8838', date: '15.01 10:15', user: 'Ольга Н.', author: 'А. Козлова', amount: 349, commission: 28, status: 'pending' },
                  ].map((t, i) => (
                    <tr key={i} className="border-b border-dark-border hover:bg-dark-card/50">
                      <td className="px-6 py-4 text-sm font-mono text-gray-400">{t.id}</td>
                      <td className="px-6 py-4 text-sm text-gray-400">{t.date}</td>
                      <td className="px-6 py-4 text-sm">{t.user}</td>
                      <td className="px-6 py-4 text-sm">{t.author}</td>
                      <td className="px-6 py-4 font-medium">{t.amount} ₽</td>
                      <td className="px-6 py-4 text-green-400 text-sm">{t.commission} ₽</td>
                      <td className="px-6 py-4">
                        <span className={`px-2 py-1 rounded text-xs ${t.status === 'success' ? 'bg-green-500/20 text-green-400' : t.status === 'refund' ? 'bg-red-500/20 text-red-400' : 'bg-yellow-500/20 text-yellow-400'}`}>
                          {t.status === 'success' ? 'Успешно' : t.status === 'refund' ? 'Возврат' : 'В обработке'}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {activeTab === 'moderation' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Модерация</h1>
            <div className="space-y-4">
              {[
                { id: 1, type: 'copyright', content: 'Трек "Тёмная сторона" — Алексей М.', reporter: 'Аноним', reason: 'Нарушение авторских прав', date: '15.01.2026', urgent: true },
                { id: 2, type: 'content', content: 'Видео "Studio Vlog #5" — Мария С.', reporter: 'user_234', reason: 'Неприемлемый контент', date: '14.01.2026', urgent: false },
                { id: 3, type: 'spam', content: 'Описание витрины — Новый автор', reporter: 'Система', reason: 'Спам / реклама', date: '14.01.2026', urgent: false },
              ].map(report => (
                <div key={report.id} className="glass-card rounded-xl p-6">
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="flex items-center gap-2 mb-1">
                        <span className={`px-2 py-0.5 rounded text-xs ${report.type === 'copyright' ? 'bg-red-500/20 text-red-400' : report.type === 'content' ? 'bg-yellow-500/20 text-yellow-400' : 'bg-blue-500/20 text-blue-400'}`}>
                          {report.type === 'copyright' ? 'Авторские права' : report.type === 'content' ? 'Контент' : 'Спам'}
                        </span>
                        {report.urgent && <span className="px-2 py-0.5 rounded text-xs bg-red-500/20 text-red-400">Срочно</span>}
                      </div>
                      <h3 className="font-medium">{report.content}</h3>
                      <p className="text-sm text-gray-400 mt-1">Причина: {report.reason}</p>
                      <p className="text-xs text-gray-500 mt-1">От: {report.reporter} • {report.date}</p>
                    </div>
                    <div className="flex gap-2">
                      <button className="px-3 py-1.5 rounded-lg bg-green-500/20 text-green-400 text-sm hover:bg-green-500/30 transition">
                        Подтвердить
                      </button>
                      <button className="px-3 py-1.5 rounded-lg bg-red-500/20 text-red-400 text-sm hover:bg-red-500/30 transition">
                        Отклонить
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'settings' && (
          <div>
            <h1 className="text-3xl font-bold mb-8">Настройки платформы</h1>
            <div className="space-y-6 max-w-2xl">
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Комиссия платформы</h3>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm text-gray-400 mb-1 block">Процент комиссии (%)</label>
                    <input type="number" defaultValue={8} className="w-full px-4 py-2.5 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white" />
                  </div>
                  <p className="text-xs text-gray-500">Комиссия взимается с каждой транзакции автора</p>
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Платёжные системы</h3>
                <div className="space-y-3">
                  {[
                    { name: 'ЮKassa', status: 'active' },
                    { name: 'CloudPayments', status: 'active' },
                    { name: 'Prodamus', status: 'inactive' },
                    { name: 'СБП', status: 'active' },
                  ].map((ps, i) => (
                    <div key={i} className="flex items-center justify-between py-2">
                      <span className="font-medium text-sm">{ps.name}</span>
                      <span className={`px-2 py-1 rounded text-xs ${ps.status === 'active' ? 'bg-green-500/20 text-green-400' : 'bg-gray-500/20 text-gray-400'}`}>
                        {ps.status === 'active' ? 'Подключена' : 'Не подключена'}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="glass-card rounded-xl p-6">
                <h3 className="text-lg font-bold mb-4">Хранилище</h3>
                <div className="space-y-3">
                  <div>
                    <div className="flex justify-between text-sm mb-1">
                      <span>Использовано</span>
                      <span className="text-gray-400">2.4 ТБ / 5 ТБ</span>
                    </div>
                    <div className="h-2 bg-dark rounded-full overflow-hidden">
                      <div className="h-full bg-primary rounded-full" style={{ width: '48%' }} />
                    </div>
                  </div>
                  <div className="text-sm text-gray-400">
                    Провайдер: Yandex Cloud Object Storage
                  </div>
                </div>
              </div>
              <button className="px-6 py-3 rounded-lg bg-primary hover:bg-primary-dark text-white font-medium transition">
                Сохранить настройки
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
