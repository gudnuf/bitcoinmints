function toggleReviewers(element) {
    const recommendationsContent = element.nextElementSibling;
    const expandIndicator = element.querySelector('.expand-indicator');
    
    if (recommendationsContent.classList.contains('expanded')) {
        recommendationsContent.classList.remove('expanded');
        expandIndicator.classList.remove('expanded');
    } else {
        recommendationsContent.classList.add('expanded');
        expandIndicator.classList.add('expanded');
    }
}


function toggleMintInfo(element) {
    const infoContent = element.nextElementSibling;
    const expandIndicator = element.querySelector('.info-expand-indicator');
    
    if (infoContent.classList.contains('expanded')) {
        infoContent.classList.remove('expanded');
        expandIndicator.classList.remove('expanded');
    } else {
        infoContent.classList.add('expanded');
        expandIndicator.classList.add('expanded');
    }
}

function filterMints(type) {
    const url = new URL(window.location);
    
    // Clear mint-type-specific filters when switching types
    url.searchParams.delete('minting');
    url.searchParams.delete('melting');
    url.searchParams.delete('nuts');
    url.searchParams.delete('modules');
    
    if (type === 'all') {
        url.searchParams.delete('type');
    } else {
        url.searchParams.set('type', type);
    }
    
    window.location.href = url.toString();
}

function toggleCashuFilters() {
    const cashuFilters = document.getElementById('cashu-filters');
    if (!cashuFilters) return; // Exit early if element doesn't exist
    
    const filterButtons = document.querySelectorAll('.filter-btn');
    let cashuBtn = null;
    
    for (let i = 0; i < filterButtons.length; i++) {
        if (filterButtons[i].textContent.includes('Cashu')) {
            cashuBtn = filterButtons[i];
            break;
        }
    }
    
    if (cashuBtn && cashuBtn.classList.contains('active')) {
        cashuFilters.classList.add('active');
    } else {
        cashuFilters.classList.remove('active');
    }
}

function toggleFedimintFilters() {
    const fedimintFilters = document.getElementById('fedimint-filters');
    if (!fedimintFilters) return; // Exit early if element doesn't exist
    
    const filterButtons = document.querySelectorAll('.filter-btn');
    let fedimintBtn = null;
    
    for (let i = 0; i < filterButtons.length; i++) {
        if (filterButtons[i].textContent.includes('Fedimint')) {
            fedimintBtn = filterButtons[i];
            break;
        }
    }
    
    if (fedimintBtn && fedimintBtn.classList.contains('active')) {
        fedimintFilters.classList.add('active');
    } else {
        fedimintFilters.classList.remove('active');
    }
}

function updateCashuFilters() {
    const url = new URL(window.location);
    const mintFilters = Array.from(document.querySelectorAll('.mint-filter:checked')).map(cb => cb.dataset.currency);
    const meltFilters = Array.from(document.querySelectorAll('.melt-filter:checked')).map(cb => cb.dataset.currency);
    const nutFilters = Array.from(document.querySelectorAll('.nut-filter:checked')).map(cb => cb.dataset.nut);
    
    if (mintFilters.length > 0) {
        url.searchParams.set('minting', mintFilters.join(','));
    } else {
        url.searchParams.delete('minting');
    }
    
    if (meltFilters.length > 0) {
        url.searchParams.set('melting', meltFilters.join(','));
    } else {
        url.searchParams.delete('melting');
    }
    
    if (nutFilters.length > 0) {
        url.searchParams.set('nuts', nutFilters.join(','));
    } else {
        url.searchParams.delete('nuts');
    }
    
    window.location.href = url.toString();
}

function clearCashuFilters() {
    const url = new URL(window.location);
    url.searchParams.delete('minting');
    url.searchParams.delete('melting');
    url.searchParams.delete('nuts');
    window.location.href = url.toString();
}

function selectAllCashuFilters() {
    const checkboxes = document.querySelectorAll('.filter-checkbox');
    checkboxes.forEach(cb => cb.checked = true);
    updateCashuFilters();
}

function updateFedimintFilters() {
    const url = new URL(window.location);
    const moduleFilters = Array.from(document.querySelectorAll('.module-filter:checked')).map(cb => cb.dataset.module);
    
    if (moduleFilters.length > 0) {
        url.searchParams.set('modules', moduleFilters.join(','));
    } else {
        url.searchParams.delete('modules');
    }
    
    window.location.href = url.toString();
}

function clearFedimintFilters() {
    const url = new URL(window.location);
    url.searchParams.delete('modules');
    window.location.href = url.toString();
}

function selectAllFedimintFilters() {
    const checkboxes = document.querySelectorAll('.module-filter');
    checkboxes.forEach(cb => cb.checked = true);
    updateFedimintFilters();
}

// Rating filter functionality
function filterByRating(minRating) {
    const ratingValue = parseFloat(minRating);
    const ratingDisplay = document.getElementById('rating-value');
    const ratingLabel = document.getElementById('rating-label');
    const displayContainer = document.getElementById('rating-display');
    const slider = document.getElementById('rating-slider');
    
    // Update display and styling based on active/inactive state
    if (ratingValue === 0) {
        // Inactive state - show all mints
        ratingDisplay.textContent = 'Show All';
        ratingLabel.textContent = '(sorted by rating)';
        displayContainer.classList.remove('rating-active');
        displayContainer.classList.add('rating-inactive');
        slider.classList.remove('slider-active');
        slider.classList.add('slider-inactive');
    } else {
        // Active state - filtering by rating
        ratingDisplay.textContent = `${ratingValue.toFixed(1)}+`;
        ratingLabel.textContent = 'stars minimum';
        displayContainer.classList.remove('rating-inactive');
        displayContainer.classList.add('rating-active');
        slider.classList.remove('slider-inactive');
        slider.classList.add('slider-active');
    }
    
    // Get all mint cards
    const mintCards = document.querySelectorAll('.mint-card');
    let visibleCount = 0;
    
    mintCards.forEach(card => {
        const ratingElement = card.querySelector('.rating-score');
        let shouldShow = true;
        
        if (ratingValue > 0) {
            // Active filtering - only show mints with ratings >= minRating
            if (ratingElement) {
                // Extract rating from the text content (format: "⭐ X.X")
                const ratingText = ratingElement.textContent.trim();
                const ratingMatch = ratingText.match(/⭐\s*([\d.]+)/);
                
                if (ratingMatch) {
                    const cardRating = parseFloat(ratingMatch[1]);
                    shouldShow = cardRating >= ratingValue;
                } else {
                    shouldShow = false; // No rating found
                }
            } else {
                shouldShow = false; // No rating section found
            }
        } else {
            // Inactive state - show all mints (already sorted by rating in backend)
            shouldShow = true;
        }
        
        if (shouldShow) {
            card.style.display = 'block';
            visibleCount++;
        } else {
            card.style.display = 'none';
        }
    });
    
    // Update visibility stats
    updateVisibleStats(visibleCount, ratingValue === 0);
}

function resetRatingFilter() {
    const slider = document.getElementById('rating-slider');
    slider.value = 0;
    filterByRating(0);
}

function updateVisibleStats(visibleCount, isInactive = false) {
    // Show/hide the "no matches" message based on visible count
    const noMatchesMessage = document.getElementById('no-rating-matches');
    const mintsGrid = document.getElementById('mints-grid');
    
    if (visibleCount === 0 && !isInactive) {
        // Only show "no matches" when actively filtering and no results
        if (noMatchesMessage) noMatchesMessage.style.display = 'block';
        if (mintsGrid) mintsGrid.style.display = 'none';
    } else {
        // Show the grid when inactive or when there are matches
        if (noMatchesMessage) noMatchesMessage.style.display = 'none';
        if (mintsGrid) mintsGrid.style.display = 'grid';
    }
}

// Hover effects for interactive elements
function setHoverStyle(element, hoverColor) {
    element.style.backgroundColor = hoverColor;
}

function clearHoverStyle(element) {
    element.style.backgroundColor = '';
}

// Initialize filters on page load
document.addEventListener('DOMContentLoaded', function() {
    toggleCashuFilters();
    toggleFedimintFilters();
    
    // Initialize rating filter
    const ratingSlider = document.getElementById('rating-slider');
    if (ratingSlider) {
        filterByRating(ratingSlider.value);
    }
    
    // Restore filter state from URL
    const url = new URL(window.location);
    const mintingCurrencies = url.searchParams.get('minting')?.split(',') || [];
    const meltingCurrencies = url.searchParams.get('melting')?.split(',') || [];
    const nutProtocols = url.searchParams.get('nuts')?.split(',') || [];
    const moduleIds = url.searchParams.get('modules')?.split(',') || [];
    
    mintingCurrencies.forEach(currency => {
        const checkbox = document.querySelector(`.mint-filter[data-currency="${currency}"]`);
        if (checkbox) checkbox.checked = true;
    });
    
    meltingCurrencies.forEach(currency => {
        const checkbox = document.querySelector(`.melt-filter[data-currency="${currency}"]`);
        if (checkbox) checkbox.checked = true;
    });
    
    nutProtocols.forEach(nut => {
        const checkbox = document.querySelector(`.nut-filter[data-nut="${nut}"]`);
        if (checkbox) checkbox.checked = true;
    });

    moduleIds.forEach(module => {
        const checkbox = document.querySelector(`.module-filter[data-module="${module}"]`);
        if (checkbox) checkbox.checked = true;
    });
}); 