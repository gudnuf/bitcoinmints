/// Shared CSS styles used across the application
pub fn get_shared_styles() -> &'static str {
    "
    :root {
        --primary-orange: #FF6B35;
        --secondary-orange: #F7931A;
        --primary-blue: #1E40AF;
        --light-blue: #3B82F6;
        --dark-blue: #1E3A8A;
        --glass-bg: rgba(0, 0, 0, 0.4);
        --glass-border: rgba(255, 255, 255, 0.12);
        --glass-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.45);
        --text-primary: #FFFFFF;
        --text-secondary: #E5E7EB;
        --text-light: #D1D5DB;
        --accent-green: #10B981;
        --accent-red: #EF4444;
        --accent-yellow: #F59E0B;
        --accent-purple: #8B5CF6;
        
        /* iOS Safe Area Support */
        --safe-area-inset-top: env(safe-area-inset-top);
        --safe-area-inset-right: env(safe-area-inset-right);
        --safe-area-inset-bottom: env(safe-area-inset-bottom);
        --safe-area-inset-left: env(safe-area-inset-left);
    }

    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
        /* iOS Safari optimizations */
        -webkit-tap-highlight-color: transparent;
        -webkit-touch-callout: none;
        -webkit-user-select: none;
        user-select: none;
    }

    /* Allow text selection for specific elements */
    p, span, div.review-content, div.mint-description, 
    div.reviewer-about, div.info-value, div.metadata-value {
        -webkit-user-select: text;
        user-select: text;
    }

    html {
        /* Prevent zoom on iOS */
        -webkit-text-size-adjust: 100%;
        /* Smooth scrolling */
        scroll-behavior: smooth;
        /* iOS momentum scrolling */
        -webkit-overflow-scrolling: touch;
    }

    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        line-height: 1.6;
        color: var(--text-primary);
        background: linear-gradient(135deg, #1e293b 0%, #334155 25%, #475569 50%, #64748b 75%, #94a3b8 100%);
        min-height: 100vh;
        min-height: calc(100vh + var(--safe-area-inset-bottom));
        position: relative;
        overflow-x: hidden;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
        /* iOS specific */
        -webkit-overflow-scrolling: touch;
        /* Prevent bounce scrolling on iOS */
        overscroll-behavior: none;
    }

    body::before {
        content: '';
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: 
            radial-gradient(circle at 20% 30%, rgba(255, 107, 53, 0.4) 0%, transparent 60%),
            radial-gradient(circle at 80% 20%, rgba(30, 64, 175, 0.4) 0%, transparent 60%),
            radial-gradient(circle at 40% 80%, rgba(59, 130, 246, 0.3) 0%, transparent 60%),
            radial-gradient(circle at 90% 70%, rgba(139, 92, 246, 0.3) 0%, transparent 60%);
        pointer-events: none;
        z-index: -1;
    }

    .dashboard-container {
        max-width: 1400px;
        margin: 0 auto;
        padding: max(1rem, var(--safe-area-inset-top)) max(1rem, var(--safe-area-inset-right)) max(1rem, var(--safe-area-inset-bottom)) max(1rem, var(--safe-area-inset-left));
        position: relative;
        z-index: 1;
    }

    .header {
        text-align: center;
        margin-bottom: 2rem;
    }

    .header-glass {
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 1px solid var(--glass-border);
        border-radius: 24px;
        padding: 1.5rem;
        box-shadow: var(--glass-shadow);
        margin-bottom: 1.5rem;
        /* iOS specific styling */
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
    }

    .logo {
        display: flex;
        justify-content: center;
        align-items: center;
        margin-bottom: 0.5rem;
    }

    .logo-image {
        height: 3rem;
        width: auto;
        filter: drop-shadow(0 4px 12px rgba(255, 107, 53, 0.4));
        transition: all 0.3s ease;
        /* iOS optimization */
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
    }

    .logo-image:hover {
        filter: drop-shadow(0 6px 20px rgba(255, 107, 53, 0.6));
        transform: scale(1.05) translateZ(0);
    }

    .tagline {
        font-size: 1rem;
        color: var(--text-primary);
        font-weight: 500;
        margin-bottom: 1.5rem;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
    }

    .nav-container {
        display: flex;
        justify-content: center;
        gap: 0.75rem;
        margin-bottom: 1.5rem;
        flex-wrap: wrap;
    }

    .nav-link {
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        color: white;
        text-decoration: none;
        padding: 0.875rem 1.75rem;
        border-radius: 50px;
        font-weight: 600;
        font-size: 0.95rem;
        transition: all 0.3s ease;
        position: relative;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        overflow: hidden;
        min-width: 120px;
        text-align: center;
        /* iOS touch optimization */
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
        cursor: pointer;
        /* iOS tap target size */
        min-height: 44px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .nav-link::before {
        content: '';
        position: absolute;
        top: 0;
        left: -100%;
        width: 100%;
        height: 100%;
        background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
        transition: left 0.5s;
    }

    .nav-link:hover::before,
    .nav-link:active::before {
        left: 100%;
    }

    .nav-link:hover,
    .nav-link:active {
        transform: translateY(-3px) translateZ(0);
        box-shadow: 0 10px 30px rgba(255, 107, 53, 0.4);
        border-color: var(--primary-orange);
        background: rgba(255, 107, 53, 0.3);
    }

    .nav-link.active {
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        border-color: var(--primary-orange);
        box-shadow: 0 8px 25px rgba(255, 107, 53, 0.5);
        transform: translateY(-2px) translateZ(0);
    }

    .stats-row {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
        margin-bottom: 2rem;
    }

    .stat-card {
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid var(--glass-border);
        border-radius: 20px;
        padding: 1.25rem;
        box-shadow: var(--glass-shadow);
        text-align: center;
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
        /* iOS optimization */
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
    }

    .stat-card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: linear-gradient(90deg, var(--primary-orange), var(--light-blue), var(--accent-purple));
    }

    .stat-card:hover {
        transform: translateY(-8px) translateZ(0);
        box-shadow: 0 16px 50px rgba(31, 38, 135, 0.6);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .stat-number {
        font-size: 2.25rem;
        font-weight: 800;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange), var(--light-blue));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        margin-bottom: 0.5rem;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    }

    .stat-label {
        color: var(--text-primary);
        font-weight: 600;
        text-transform: uppercase;
        font-size: 0.85rem;
        letter-spacing: 0.75px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .empty-state {
        text-align: center;
        padding: 3rem 1.5rem;
        color: var(--text-primary);
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid var(--glass-border);
        border-radius: 24px;
        box-shadow: var(--glass-shadow);
    }

    .empty-icon {
        font-size: 3.5rem;
        margin-bottom: 1rem;
        opacity: 0.8;
    }

    /* Button and interactive element improvements for iOS */
    button, .filter-btn, .capability-checkbox, .nav-link {
        /* Prevent iOS zoom on form elements */
        font-size: 16px;
        -webkit-appearance: none;
        appearance: none;
        /* Better touch targets */
        min-height: 44px;
        min-width: 44px;
    }

    /* iOS specific scrolling improvements */
    .mints-grid, .reviews-grid {
        -webkit-overflow-scrolling: touch;
    }

    /* Mobile-first responsive design with iOS optimizations */
    @media (max-width: 480px) {
        .dashboard-container {
            padding: max(0.75rem, var(--safe-area-inset-top)) max(0.75rem, var(--safe-area-inset-right)) max(0.75rem, var(--safe-area-inset-bottom)) max(0.75rem, var(--safe-area-inset-left));
        }

        .header-glass {
            padding: 1rem;
            border-radius: 20px;
        }

        .logo-image {
            height: 2.5rem;
        }

        .tagline {
            font-size: 0.9rem;
            margin-bottom: 1rem;
        }

        .nav-container {
            gap: 0.5rem;
            margin-bottom: 1rem;
        }

        .nav-link {
            padding: 0.75rem 1.25rem;
            font-size: 16px; /* Prevent iOS zoom */
            min-width: 100px;
        }

        .stats-row {
            grid-template-columns: repeat(2, 1fr);
            gap: 0.75rem;
            margin-bottom: 1.5rem;
        }

        .stat-card {
            padding: 1rem;
            border-radius: 16px;
        }

        .stat-number {
            font-size: 1.75rem;
        }

        .stat-label {
            font-size: 0.75rem;
        }

        .empty-state {
            padding: 2rem 1rem;
            border-radius: 20px;
        }

        .empty-icon {
            font-size: 2.5rem;
        }

        /* iOS specific mobile adjustments */
        body {
            /* Account for iOS Safari's dynamic viewport */
            min-height: 100dvh;
        }
    }

    @media (min-width: 481px) and (max-width: 768px) {
        .dashboard-container {
            padding: max(1rem, var(--safe-area-inset-top)) max(1rem, var(--safe-area-inset-right)) max(1rem, var(--safe-area-inset-bottom)) max(1rem, var(--safe-area-inset-left));
        }

        .stats-row {
            grid-template-columns: repeat(2, 1fr);
            gap: 1rem;
        }

        .logo-image {
            height: 3rem;
        }

        .tagline {
            font-size: 1rem;
        }

        .nav-container {
            gap: 0.75rem;
        }
    }

    @media (min-width: 769px) {
        .dashboard-container {
            padding: max(2rem, var(--safe-area-inset-top)) max(2rem, var(--safe-area-inset-right)) max(2rem, var(--safe-area-inset-bottom)) max(2rem, var(--safe-area-inset-left));
        }

        .header {
            margin-bottom: 3rem;
        }

        .header-glass {
            padding: 2rem;
            margin-bottom: 2rem;
        }

        .logo-image {
            height: 4rem;
        }

        .tagline {
            font-size: 1.2rem;
            margin-bottom: 2rem;
        }

        .nav-container {
            gap: 1rem;
            margin-bottom: 2rem;
        }

        .stats-row {
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 3rem;
        }

        .stat-card {
            padding: 1.5rem;
        }

        .stat-number {
            font-size: 2.5rem;
        }

        .empty-state {
            padding: 4rem 2rem;
        }

        .empty-icon {
            font-size: 4rem;
        }
    }

    /* iOS Safari specific fixes */
    @supports (-webkit-touch-callout: none) {
        /* iOS Safari specific styles */
        .dashboard-container {
            /* Better handling of Safari's UI elements */
            padding-top: max(1rem, env(safe-area-inset-top));
            padding-bottom: max(1rem, env(safe-area-inset-bottom));
        }
        
        /* Fix for iOS Safari viewport issues */
        body {
            min-height: -webkit-fill-available;
        }
        
        html {
            height: -webkit-fill-available;
        }
    }
    "
}
