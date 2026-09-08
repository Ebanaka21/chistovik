import { useState, useRef, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Play, Pause, Lock, Music, Video, Heart, Share2, ExternalLink, SkipBack, SkipForward, Volume2 } from 'lucide-react';

const AUTHOR = {
  name: 'Алексей Морозов',
  slug: 'aleksei-morozov',
  avatar: '🎹',
  cover: '',
  description: 'Электронный музыкант и продюсер. Эксклюзивные треки в Lossless качестве, которых нет на других площадках.',
  socials: { telegram: '@morozov_music', vk: 'morozov_music', youtube: 'AlekseiMorozov' },
  subscribers: 1247,
};

const TRACKS = [
  { id: 1, title: 'Ночной город', album: 'Сигналы', duration: '4:23', type: 'audio' as const, format: 'FLAC', cover: '🌃' },
  { id: 2, title: 'Рассвет', album: 'Сигналы', duration: '3:45', type: 'audio' as const, format: 'WAV', cover: '🌅' },
  { id: 3, title: 'Пульс', album: 'Импульс', duration: '5:12', type: 'audio' as const, format: 'FLAC', cover: '💓' },
  { id: 4, title: 'Тишина', album: 'Импульс', duration: '6:01', type: 'audio' as const, format: 'FLAC', cover: '🤫' },
  { id: 5, title: 'Эхо', album: 'Отражения', duration: '4:55', type: 'audio' as const, format: 'MP3 320', cover: '🔊' },
];

const VIDEOS = [
  { id: 1, title: 'Making of: Сигналы', duration: '12:34', type: 'video' as const, cover: '🎬' },
  { id: 2, title: 'Live Session (закрытый концерт)', duration: '45:20', type: 'video' as const, cover: '🎤' },
  { id: 3, title: 'Studio Vlog #3', duration: '8:15', type: 'video' as const, cover: '📹' },
];

const PLANS = [
  { id: 'music', title: 'Музыка', icon: '🎵', price: 199, period: 'мес', description: 'Все треки в Lossless', features: ['FLAC / WAV / MP3 320', 'Все релизы', 'Ранний доступ'] },
  { id: 'video', title: 'Видео', icon: '🎬', price: 249, period: 'мес', description: 'Эксклюзивные видео', features: ['HD / 4K качество', 'Behind the scenes', 'Live записи'] },
  { id: 'combo', title: 'Комбо', icon: '🎁', price: 349, period: 'мес', description: 'Музыка + Видео', features: ['Всё из Музыка', 'Всё из Видео', 'Скидка 20%'], popular: true },
];

export default function ShowcasePage() {
  const [activeTab, setActiveTab] = useState<'all' | 'audio' | 'video'>('all');
  const [playingTrack, setPlayingTrack] = useState<number | null>(null);
  const [isTeaserMode] = useState(true);
  const [teaserProgress, setTeaserProgress] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [showPaywall, setShowPaywall] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (isPlaying && playingTrack) {
      intervalRef.current = setInterval(() => {
        setTeaserProgress(prev => {
          if (prev >= 100) {
            setIsPlaying(false);
            setShowPaywall(true);
            return 100;
          }
          return prev + (100 / 15); // 15 seconds teaser
        });
      }, 1000);
    }
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [isPlaying, playingTrack]);

  const handlePlay = (trackId: number) => {
    if (playingTrack === trackId) {
      setIsPlaying(!isPlaying);
    } else {
      setPlayingTrack(trackId);
      setTeaserProgress(0);
      setIsPlaying(true);
      setShowPaywall(false);
    }
  };

  const filteredTracks = activeTab === 'all' ? [...TRACKS, ...VIDEOS] :
    activeTab === 'audio' ? TRACKS : VIDEOS;

  return (
    <div className="min-h-screen bg-dark">
      {/* Header */}
      <header className="fixed top-0 w-full z-50 glass-card border-b border-dark-border">
        <div className="max-w-7xl mx-auto px-4 py-3 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <Music className="w-5 h-5 text-white" />
            </div>
            <span className="text-xl font-bold gradient-text">Чистовик</span>
          </Link>
          <div className="flex items-center gap-3">
            <Link to="/login" className="px-4 py-2 rounded-lg bg-primary hover:bg-primary-dark text-white text-sm transition">
              Подписаться
            </Link>
          </div>
        </div>
      </header>

      {/* Author Header */}
      <section className="pt-24 pb-8 px-4 relative">
        <div className="absolute inset-0 bg-gradient-to-b from-primary/10 to-transparent" />
        <div className="max-w-5xl mx-auto relative z-10">
          <div className="flex flex-col md:flex-row items-center md:items-end gap-6">
            <div className="w-32 h-32 rounded-2xl bg-dark-card flex items-center justify-center text-6xl border-2 border-primary/30">
              {AUTHOR.avatar}
            </div>
            <div className="text-center md:text-left flex-1">
              <h1 className="text-4xl font-bold mb-2">{AUTHOR.name}</h1>
              <p className="text-gray-400 mb-3">{AUTHOR.description}</p>
              <div className="flex items-center gap-4 justify-center md:justify-start">
                <span className="text-sm text-gray-400">{AUTHOR.subscribers} подписчиков</span>
                <div className="flex items-center gap-3">
                  <a href="#" className="text-gray-400 hover:text-primary-light transition flex items-center gap-1 text-sm">
                    <ExternalLink className="w-3 h-3" /> Telegram
                  </a>
                  <a href="#" className="text-gray-400 hover:text-primary-light transition flex items-center gap-1 text-sm">
                    <ExternalLink className="w-3 h-3" /> VK
                  </a>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Pricing Cards */}
      <section className="py-8 px-4">
        <div className="max-w-5xl mx-auto">
          <h2 className="text-2xl font-bold mb-6">Тарифы подписки</h2>
          <div className="grid md:grid-cols-3 gap-4">
            {PLANS.map(plan => (
              <div key={plan.id} className={`glass-card rounded-2xl p-6 relative ${plan.popular ? 'border-primary-light glow-purple' : ''}`}>
                {plan.popular && (
                  <div className="absolute -top-3 right-4 px-3 py-1 bg-primary rounded-full text-xs font-bold">
                    Популярный
                  </div>
                )}
                <div className="text-3xl mb-3">{plan.icon}</div>
                <h3 className="text-lg font-bold mb-1">{plan.title}</h3>
                <p className="text-gray-400 text-sm mb-3">{plan.description}</p>
                <div className="text-2xl font-bold mb-4">
                  {plan.price} ₽<span className="text-sm text-gray-400 font-normal">/{plan.period}</span>
                </div>
                <ul className="text-sm text-gray-300 space-y-1 mb-4">
                  {plan.features.map((f, i) => (
                    <li key={i} className="flex items-center gap-2">✓ {f}</li>
                  ))}
                </ul>
                <Link to="/login" className={`block text-center py-2.5 rounded-lg font-medium transition ${plan.popular ? 'bg-primary hover:bg-primary-dark text-white' : 'bg-dark border border-dark-border hover:border-primary-light text-white'}`}>
                  Подписаться
                </Link>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Content Section */}
      <section className="py-8 px-4 pb-32">
        <div className="max-w-5xl mx-auto">
          <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold">Контент</h2>
            <div className="flex gap-2">
              {(['all', 'audio', 'video'] as const).map(tab => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition ${activeTab === tab ? 'bg-primary text-white' : 'glass-card text-gray-300 hover:text-white'}`}
                >
                  {tab === 'all' ? 'Все' : tab === 'audio' ? '🎵 Музыка' : '🎬 Видео'}
                </button>
              ))}
            </div>
          </div>

          <div className="space-y-3">
            {filteredTracks.map(item => (
              <div key={`${item.type}-${item.id}`} className="glass-card rounded-xl p-4 hover:border-primary-light/50 transition group">
                <div className="flex items-center gap-4">
                  {/* Cover */}
                  <div className="w-14 h-14 rounded-lg bg-dark flex items-center justify-center text-2xl flex-shrink-0">
                    {item.cover}
                  </div>
                  
                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <h3 className="font-medium truncate">{item.title}</h3>
                      {item.type === 'audio' && (
                        <span className="px-2 py-0.5 rounded text-xs bg-primary/20 text-primary-light">
                          {(item as any).format}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 text-sm text-gray-400">
                      {item.type === 'audio' && <span>{(item as any).album}</span>}
                      <span>{item.duration}</span>
                      {item.type === 'video' && <span className="flex items-center gap-1"><Video className="w-3 h-3" /> Видео</span>}
                      {item.type === 'audio' && <span className="flex items-center gap-1"><Music className="w-3 h-3" /> Аудио</span>}
                    </div>
                  </div>

                  {/* Play / Lock */}
                  <div className="flex items-center gap-3">
                    {isTeaserMode && playingTrack === item.id && item.type === 'audio' ? (
                      <div className="flex items-center gap-2">
                        <div className="w-24 h-1.5 bg-dark rounded-full overflow-hidden">
                          <div className="h-full bg-primary rounded-full transition-all" style={{ width: `${teaserProgress}%` }} />
                        </div>
                        <span className="text-xs text-gray-400">Тизер</span>
                      </div>
                    ) : null}
                    
                    {item.type === 'audio' ? (
                      <button
                        onClick={() => handlePlay(item.id)}
                        className="w-10 h-10 rounded-full bg-primary/20 hover:bg-primary/40 flex items-center justify-center transition"
                      >
                        {playingTrack === item.id && isPlaying ? (
                          <Pause className="w-4 h-4 text-primary-light" />
                        ) : (
                          <Play className="w-4 h-4 text-primary-light ml-0.5" />
                        )}
                      </button>
                    ) : (
                      <button
                        onClick={() => setShowPaywall(true)}
                        className="w-10 h-10 rounded-full bg-primary/20 hover:bg-primary/40 flex items-center justify-center transition"
                      >
                        <Play className="w-4 h-4 text-primary-light ml-0.5" />
                      </button>
                    )}
                    
                    <Lock className="w-4 h-4 text-gray-500" />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Paywall Modal */}
      {showPaywall && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm" onClick={() => setShowPaywall(false)}>
          <div className="glass-card rounded-2xl p-8 max-w-md w-full text-center" onClick={e => e.stopPropagation()}>
            <div className="text-5xl mb-4">🔒</div>
            <h3 className="text-2xl font-bold mb-2">Полный доступ закрыт</h3>
            <p className="text-gray-400 mb-6">Оформите подписку, чтобы слушать и смотреть полный контент {AUTHOR.name}</p>
            <Link to="/login" className="block w-full py-3 rounded-xl bg-primary hover:bg-primary-dark text-white font-semibold transition mb-3">
              Оформить подписку
            </Link>
            <button onClick={() => setShowPaywall(false)} className="text-gray-400 hover:text-white text-sm transition">
              Закрыть
            </button>
          </div>
        </div>
      )}

      {/* Audio Player Bar */}
      {playingTrack && (
        <div className="fixed bottom-0 left-0 right-0 glass-card border-t border-dark-border z-40">
          <div className="max-w-5xl mx-auto px-4 py-3 flex items-center gap-4">
            <div className="w-10 h-10 rounded bg-dark flex items-center justify-center text-lg">
              {TRACKS.find(t => t.id === playingTrack)?.cover}
            </div>
            <div className="flex-1 min-w-0">
              <div className="font-medium text-sm truncate">{TRACKS.find(t => t.id === playingTrack)?.title}</div>
              <div className="text-xs text-gray-400">Тизер • 15 сек</div>
            </div>
            <div className="flex items-center gap-3">
              <button className="text-gray-400 hover:text-white"><SkipBack className="w-4 h-4" /></button>
              <button onClick={() => setIsPlaying(!isPlaying)} className="w-8 h-8 rounded-full bg-primary flex items-center justify-center">
                {isPlaying ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4 ml-0.5" />}
              </button>
              <button className="text-gray-400 hover:text-white"><SkipForward className="w-4 h-4" /></button>
              <Volume2 className="w-4 h-4 text-gray-400" />
            </div>
          </div>
          {/* Progress bar */}
          <div className="h-1 bg-dark-border">
            <div className="h-full bg-primary transition-all" style={{ width: `${teaserProgress}%` }} />
          </div>
        </div>
      )}
    </div>
  );
}
