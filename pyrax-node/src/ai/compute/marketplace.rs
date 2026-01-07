//! AI Marketplace
//!
//! Decentralized marketplace for AI models and compute services

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::types::{Address, H256};
use super::PLATFORM_FEE_PERCENT;

/// Listing type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ListingType {
    /// Model for sale/license
    Model,
    /// Compute capacity
    Compute,
    /// Dataset
    Dataset,
    /// Fine-tuning service
    FineTuning,
    /// Custom service
    Custom,
}

/// Pricing model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingModel {
    /// Fixed price
    Fixed,
    /// Per inference/request
    PerRequest,
    /// Per compute hour
    PerHour,
    /// Per token (for LLMs)
    PerToken,
    /// Subscription
    Subscription,
    /// Auction
    Auction,
}

/// Listing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListingStatus {
    /// Draft, not visible
    Draft,
    /// Active and visible
    Active,
    /// Paused by seller
    Paused,
    /// Sold out
    SoldOut,
    /// Expired
    Expired,
    /// Removed
    Removed,
}

/// Marketplace listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    /// Unique listing ID
    pub id: H256,
    /// Seller address
    pub seller: Address,
    /// Listing type
    pub listing_type: ListingType,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Reference ID (model ID, node address, etc.)
    pub reference_id: H256,
    /// Pricing model
    pub pricing: PricingModel,
    /// Base price (PYRAX smallest unit)
    pub price: u64,
    /// Minimum purchase quantity
    pub min_quantity: u64,
    /// Maximum purchase quantity (0 = unlimited)
    pub max_quantity: u64,
    /// Available quantity (for limited items)
    pub available: Option<u64>,
    /// Status
    pub status: ListingStatus,
    /// Tags for search
    pub tags: Vec<String>,
    /// Preview/thumbnail hash
    pub preview_hash: Option<H256>,
    /// Terms and conditions
    pub terms: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Expiration timestamp (0 = no expiry)
    pub expires_at: u64,
    /// Total sales count
    pub sales_count: u64,
    /// Total revenue (PYRAX)
    pub total_revenue: u64,
    /// Average rating (0-100)
    pub rating: u32,
    /// Rating count
    pub rating_count: u32,
    /// Is featured
    pub featured: bool,
}

impl Listing {
    /// Create new listing
    pub fn new(
        seller: Address,
        listing_type: ListingType,
        title: String,
        description: String,
        reference_id: H256,
        pricing: PricingModel,
        price: u64,
    ) -> Self {
        let id = Self::generate_id(&seller, &title);
        
        Self {
            id,
            seller,
            listing_type,
            title,
            description,
            reference_id,
            pricing,
            price,
            min_quantity: 1,
            max_quantity: 0,
            available: None,
            status: ListingStatus::Draft,
            tags: Vec::new(),
            preview_hash: None,
            terms: None,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
            expires_at: 0,
            sales_count: 0,
            total_revenue: 0,
            rating: 0,
            rating_count: 0,
            featured: false,
        }
    }

    /// Generate listing ID
    fn generate_id(seller: &Address, title: &str) -> H256 {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(&seller.0);
        hasher.update(title.as_bytes());
        hasher.update(&current_timestamp().to_le_bytes());
        H256::from_slice(hasher.finalize().as_bytes())
    }

    /// Activate listing
    pub fn activate(&mut self) {
        self.status = ListingStatus::Active;
        self.updated_at = current_timestamp();
    }

    /// Pause listing
    pub fn pause(&mut self) {
        if self.status == ListingStatus::Active {
            self.status = ListingStatus::Paused;
            self.updated_at = current_timestamp();
        }
    }

    /// Resume listing
    pub fn resume(&mut self) {
        if self.status == ListingStatus::Paused {
            self.status = ListingStatus::Active;
            self.updated_at = current_timestamp();
        }
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        self.expires_at > 0 && current_timestamp() > self.expires_at
    }

    /// Record sale
    pub fn record_sale(&mut self, quantity: u64, revenue: u64) {
        self.sales_count += quantity;
        self.total_revenue += revenue;
        self.updated_at = current_timestamp();

        if let Some(ref mut avail) = self.available {
            *avail = avail.saturating_sub(quantity);
            if *avail == 0 {
                self.status = ListingStatus::SoldOut;
            }
        }
    }

    /// Add rating
    pub fn add_rating(&mut self, rating: u32) {
        let total = self.rating as u64 * self.rating_count as u64 + rating as u64;
        self.rating_count += 1;
        self.rating = (total / self.rating_count as u64) as u32;
        self.updated_at = current_timestamp();
    }

    /// Update price
    pub fn update_price(&mut self, price: u64) {
        self.price = price;
        self.updated_at = current_timestamp();
    }

    /// Calculate total price for quantity
    pub fn calculate_price(&self, quantity: u64) -> u64 {
        self.price * quantity
    }
}

/// Bid for auction listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bid {
    /// Bid ID
    pub id: H256,
    /// Listing ID
    pub listing_id: H256,
    /// Bidder address
    pub bidder: Address,
    /// Bid amount
    pub amount: u64,
    /// Bid timestamp
    pub created_at: u64,
    /// Is winning bid
    pub is_winning: bool,
    /// Is cancelled
    pub cancelled: bool,
}

impl Bid {
    /// Create new bid
    pub fn new(listing_id: H256, bidder: Address, amount: u64) -> Self {
        let id = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&listing_id.0);
            hasher.update(&bidder.0);
            hasher.update(&amount.to_le_bytes());
            hasher.update(&current_timestamp().to_le_bytes());
            H256::from_slice(hasher.finalize().as_bytes())
        };

        Self {
            id,
            listing_id,
            bidder,
            amount,
            created_at: current_timestamp(),
            is_winning: false,
            cancelled: false,
        }
    }
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order created, awaiting payment
    Pending,
    /// Payment received
    Paid,
    /// Order being processed
    Processing,
    /// Order completed
    Completed,
    /// Order cancelled
    Cancelled,
    /// Order refunded
    Refunded,
    /// Order disputed
    Disputed,
}

/// Purchase order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Order ID
    pub id: H256,
    /// Listing ID
    pub listing_id: H256,
    /// Buyer address
    pub buyer: Address,
    /// Seller address
    pub seller: Address,
    /// Quantity purchased
    pub quantity: u64,
    /// Total price (before fees)
    pub subtotal: u64,
    /// Platform fee
    pub platform_fee: u64,
    /// Total paid
    pub total: u64,
    /// Order status
    pub status: OrderStatus,
    /// Creation timestamp
    pub created_at: u64,
    /// Payment timestamp
    pub paid_at: Option<u64>,
    /// Completion timestamp
    pub completed_at: Option<u64>,
    /// Delivery hash (if applicable)
    pub delivery_hash: Option<H256>,
    /// Buyer rating (after completion)
    pub rating: Option<u32>,
    /// Buyer review
    pub review: Option<String>,
}

impl Order {
    /// Create new order
    pub fn new(listing: &Listing, buyer: Address, quantity: u64) -> Self {
        let subtotal = listing.calculate_price(quantity);
        let platform_fee = (subtotal as u128 * PLATFORM_FEE_PERCENT as u128 / 100) as u64;
        let total = subtotal + platform_fee;

        let id = {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&listing.id.0);
            hasher.update(&buyer.0);
            hasher.update(&current_timestamp().to_le_bytes());
            H256::from_slice(hasher.finalize().as_bytes())
        };

        Self {
            id,
            listing_id: listing.id,
            buyer,
            seller: listing.seller,
            quantity,
            subtotal,
            platform_fee,
            total,
            status: OrderStatus::Pending,
            created_at: current_timestamp(),
            paid_at: None,
            completed_at: None,
            delivery_hash: None,
            rating: None,
            review: None,
        }
    }

    /// Mark as paid
    pub fn mark_paid(&mut self) {
        self.status = OrderStatus::Paid;
        self.paid_at = Some(current_timestamp());
    }

    /// Mark as processing
    pub fn start_processing(&mut self) {
        if self.status == OrderStatus::Paid {
            self.status = OrderStatus::Processing;
        }
    }

    /// Complete order
    pub fn complete(&mut self, delivery_hash: Option<H256>) {
        self.status = OrderStatus::Completed;
        self.completed_at = Some(current_timestamp());
        self.delivery_hash = delivery_hash;
    }

    /// Cancel order
    pub fn cancel(&mut self) {
        if self.status == OrderStatus::Pending || self.status == OrderStatus::Paid {
            self.status = OrderStatus::Cancelled;
        }
    }

    /// Refund order
    pub fn refund(&mut self) {
        self.status = OrderStatus::Refunded;
    }

    /// Add rating
    pub fn add_rating(&mut self, rating: u32, review: Option<String>) {
        self.rating = Some(rating);
        self.review = review;
    }
}

/// AI Marketplace
pub struct AIMarketplace {
    /// Listings by ID
    listings: Arc<RwLock<HashMap<H256, Listing>>>,
    /// Listings by seller
    by_seller: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Listings by type
    by_type: Arc<RwLock<HashMap<ListingType, Vec<H256>>>>,
    /// Featured listings
    featured: Arc<RwLock<Vec<H256>>>,
    /// Bids by listing
    bids: Arc<RwLock<HashMap<H256, Vec<Bid>>>>,
    /// Orders by ID
    orders: Arc<RwLock<HashMap<H256, Order>>>,
    /// Orders by buyer
    orders_by_buyer: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Orders by seller
    orders_by_seller: Arc<RwLock<HashMap<Address, Vec<H256>>>>,
    /// Total volume (PYRAX)
    total_volume: Arc<RwLock<u64>>,
    /// Total fees collected
    total_fees: Arc<RwLock<u64>>,
}

impl AIMarketplace {
    /// Create new marketplace
    pub fn new() -> Self {
        Self {
            listings: Arc::new(RwLock::new(HashMap::new())),
            by_seller: Arc::new(RwLock::new(HashMap::new())),
            by_type: Arc::new(RwLock::new(HashMap::new())),
            featured: Arc::new(RwLock::new(Vec::new())),
            bids: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            orders_by_buyer: Arc::new(RwLock::new(HashMap::new())),
            orders_by_seller: Arc::new(RwLock::new(HashMap::new())),
            total_volume: Arc::new(RwLock::new(0)),
            total_fees: Arc::new(RwLock::new(0)),
        }
    }

    /// Create listing
    pub fn create_listing(&self, listing: Listing) -> Result<H256, MarketplaceError> {
        let id = listing.id;
        let seller = listing.seller;
        let listing_type = listing.listing_type;

        {
            let mut listings = self.listings.write();
            if listings.contains_key(&id) {
                return Err(MarketplaceError::ListingExists(id));
            }

            self.by_seller.write()
                .entry(seller)
                .or_insert_with(Vec::new)
                .push(id);

            self.by_type.write()
                .entry(listing_type)
                .or_insert_with(Vec::new)
                .push(id);

            listings.insert(id, listing);
        }

        Ok(id)
    }

    /// Get listing
    pub fn get_listing(&self, id: &H256) -> Option<Listing> {
        self.listings.read().get(id).cloned()
    }

    /// Update listing
    pub fn update_listing(&self, id: &H256, f: impl FnOnce(&mut Listing)) -> Result<(), MarketplaceError> {
        let mut listings = self.listings.write();
        let listing = listings.get_mut(id)
            .ok_or_else(|| MarketplaceError::ListingNotFound(*id))?;
        f(listing);
        Ok(())
    }

    /// Activate listing
    pub fn activate_listing(&self, id: &H256, seller: &Address) -> Result<(), MarketplaceError> {
        self.update_listing(id, |l| {
            if l.seller == *seller {
                l.activate();
            }
        })
    }

    /// Get listings by seller
    pub fn get_by_seller(&self, seller: &Address) -> Vec<Listing> {
        let ids = self.by_seller.read()
            .get(seller)
            .cloned()
            .unwrap_or_default();

        let listings = self.listings.read();
        ids.iter()
            .filter_map(|id| listings.get(id).cloned())
            .collect()
    }

    /// Get listings by type
    pub fn get_by_type(&self, listing_type: ListingType) -> Vec<Listing> {
        let ids = self.by_type.read()
            .get(&listing_type)
            .cloned()
            .unwrap_or_default();

        let listings = self.listings.read();
        ids.iter()
            .filter_map(|id| listings.get(id).cloned())
            .filter(|l| l.status == ListingStatus::Active)
            .collect()
    }

    /// Search listings
    pub fn search(&self, query: &str, listing_type: Option<ListingType>) -> Vec<Listing> {
        let query_lower = query.to_lowercase();
        self.listings.read()
            .values()
            .filter(|l| {
                l.status == ListingStatus::Active &&
                listing_type.map_or(true, |t| l.listing_type == t) &&
                (l.title.to_lowercase().contains(&query_lower) ||
                 l.description.to_lowercase().contains(&query_lower) ||
                 l.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)))
            })
            .cloned()
            .collect()
    }

    /// Get featured listings
    pub fn get_featured(&self) -> Vec<Listing> {
        let ids = self.featured.read().clone();
        let listings = self.listings.read();
        ids.iter()
            .filter_map(|id| listings.get(id).cloned())
            .filter(|l| l.status == ListingStatus::Active)
            .collect()
    }

    /// Create order
    pub fn create_order(&self, listing_id: &H256, buyer: Address, quantity: u64) -> Result<Order, MarketplaceError> {
        let listing = self.listings.read()
            .get(listing_id)
            .cloned()
            .ok_or_else(|| MarketplaceError::ListingNotFound(*listing_id))?;

        if listing.status != ListingStatus::Active {
            return Err(MarketplaceError::ListingNotActive);
        }

        if quantity < listing.min_quantity {
            return Err(MarketplaceError::QuantityTooLow);
        }

        if listing.max_quantity > 0 && quantity > listing.max_quantity {
            return Err(MarketplaceError::QuantityTooHigh);
        }

        if let Some(avail) = listing.available {
            if quantity > avail {
                return Err(MarketplaceError::InsufficientStock);
            }
        }

        let order = Order::new(&listing, buyer, quantity);
        let order_id = order.id;

        self.orders.write().insert(order_id, order.clone());
        self.orders_by_buyer.write()
            .entry(buyer)
            .or_insert_with(Vec::new)
            .push(order_id);
        self.orders_by_seller.write()
            .entry(listing.seller)
            .or_insert_with(Vec::new)
            .push(order_id);

        Ok(order)
    }

    /// Pay for order
    pub fn pay_order(&self, order_id: &H256) -> Result<(), MarketplaceError> {
        let mut orders = self.orders.write();
        let order = orders.get_mut(order_id)
            .ok_or_else(|| MarketplaceError::OrderNotFound(*order_id))?;

        if order.status != OrderStatus::Pending {
            return Err(MarketplaceError::InvalidOrderStatus);
        }

        order.mark_paid();
        Ok(())
    }

    /// Complete order
    pub fn complete_order(&self, order_id: &H256, delivery_hash: Option<H256>) -> Result<(), MarketplaceError> {
        let mut orders = self.orders.write();
        let order = orders.get_mut(order_id)
            .ok_or_else(|| MarketplaceError::OrderNotFound(*order_id))?;

        order.complete(delivery_hash);

        // Update listing stats
        let listing_id = order.listing_id;
        let quantity = order.quantity;
        let revenue = order.subtotal;
        let platform_fee = order.platform_fee;

        drop(orders);

        self.update_listing(&listing_id, |l| l.record_sale(quantity, revenue))?;
        *self.total_volume.write() += revenue;
        *self.total_fees.write() += platform_fee;

        Ok(())
    }

    /// Get order
    pub fn get_order(&self, id: &H256) -> Option<Order> {
        self.orders.read().get(id).cloned()
    }

    /// Get orders by buyer
    pub fn get_orders_by_buyer(&self, buyer: &Address) -> Vec<Order> {
        let ids = self.orders_by_buyer.read()
            .get(buyer)
            .cloned()
            .unwrap_or_default();

        let orders = self.orders.read();
        ids.iter()
            .filter_map(|id| orders.get(id).cloned())
            .collect()
    }

    /// Get marketplace statistics
    pub fn stats(&self) -> MarketplaceStats {
        let listings = self.listings.read();
        
        let mut by_type = HashMap::new();
        for listing in listings.values() {
            if listing.status == ListingStatus::Active {
                *by_type.entry(listing.listing_type).or_insert(0u64) += 1;
            }
        }

        MarketplaceStats {
            total_listings: listings.len() as u64,
            active_listings: listings.values().filter(|l| l.status == ListingStatus::Active).count() as u64,
            total_orders: self.orders.read().len() as u64,
            total_volume: *self.total_volume.read(),
            total_fees: *self.total_fees.read(),
            by_type,
        }
    }
}

impl Default for AIMarketplace {
    fn default() -> Self {
        Self::new()
    }
}

/// Marketplace errors
#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("Listing not found: {0:?}")]
    ListingNotFound(H256),

    #[error("Listing already exists: {0:?}")]
    ListingExists(H256),

    #[error("Listing not active")]
    ListingNotActive,

    #[error("Order not found: {0:?}")]
    OrderNotFound(H256),

    #[error("Invalid order status")]
    InvalidOrderStatus,

    #[error("Quantity too low")]
    QuantityTooLow,

    #[error("Quantity too high")]
    QuantityTooHigh,

    #[error("Insufficient stock")]
    InsufficientStock,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Payment failed: {0}")]
    PaymentFailed(String),
}

/// Marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_listings: u64,
    pub active_listings: u64,
    pub total_orders: u64,
    pub total_volume: u64,
    pub total_fees: u64,
    pub by_type: HashMap<ListingType, u64>,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listing_creation() {
        let seller = Address([1u8; 20]);
        let listing = Listing::new(
            seller,
            ListingType::Model,
            "Test Model".to_string(),
            "A test AI model".to_string(),
            H256([1u8; 32]),
            PricingModel::PerRequest,
            100_000_000, // 1 PYRAX
        );

        assert_eq!(listing.status, ListingStatus::Draft);
        assert_eq!(listing.seller, seller);
    }

    #[test]
    fn test_order_creation() {
        let seller = Address([1u8; 20]);
        let buyer = Address([2u8; 20]);
        let mut listing = Listing::new(
            seller,
            ListingType::Model,
            "Test Model".to_string(),
            "A test AI model".to_string(),
            H256([1u8; 32]),
            PricingModel::Fixed,
            100_000_000,
        );
        listing.activate();

        let order = Order::new(&listing, buyer, 1);

        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.quantity, 1);
        assert!(order.platform_fee > 0);
    }

    #[test]
    fn test_marketplace() {
        let marketplace = AIMarketplace::new();
        let seller = Address([1u8; 20]);
        let buyer = Address([2u8; 20]);

        let mut listing = Listing::new(
            seller,
            ListingType::Model,
            "Test Model".to_string(),
            "A test AI model".to_string(),
            H256([1u8; 32]),
            PricingModel::Fixed,
            100_000_000,
        );
        listing.activate();

        let listing_id = marketplace.create_listing(listing).unwrap();

        // Create order
        let order = marketplace.create_order(&listing_id, buyer, 1).unwrap();
        assert_eq!(order.status, OrderStatus::Pending);

        // Pay order
        assert!(marketplace.pay_order(&order.id).is_ok());

        let stats = marketplace.stats();
        assert_eq!(stats.active_listings, 1);
    }
}
