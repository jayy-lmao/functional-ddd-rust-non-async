#![feature(type_alias_impl_trait)]

pub mod simple_types {
    use anyhow::{anyhow, Result};
    use lazy_static::lazy_static;

    use regex::Regex;

    lazy_static! {
        static ref EMAIL_RE: Regex = Regex::new(r".+@.+").unwrap();
    }
    pub struct String50(String);

    impl String50 {
        pub fn create(string50: String) -> Result<Self> {
            if string50.len() > 50 {
                return Err(anyhow!("String to big"));
            }
            Ok(Self(string50))
        }
    }
    pub struct EmailAddress(String);

    impl EmailAddress {
        pub fn create(email: String) -> Result<Self> {
            if EMAIL_RE.is_match(&email) {
                return Err(anyhow!("Must have @ separator"));
            }
            Ok(Self(email))
        }
    }
}

pub mod public_types {
    use crate::simple_types::*;

    pub struct UnvalidatedCustomerInfo {
        pub first_name: String,
        pub last_name: String,
        pub email_address: String,
    }

    pub struct ValidatedCustomerInfo {
        first_name: String50,
        last_name: String50,
        email_address: EmailAddress,
    }

    impl ValidatedCustomerInfo {
        pub fn new(first_name: String50, last_name: String50, email_address: EmailAddress) -> Self {
            Self {
                first_name,
                last_name,
                email_address,
            }
        }
    }
    pub struct UnvalidatedOrder {
        pub order_id: String,
        pub lines: Vec<UnvalidatedOrderLine>,
    }
    pub struct UnvalidatedOrderLine {
        pub order_line_id: String,
        pub product_code: String,
        pub quantity: i64,
    }
    pub struct Address {}

    pub struct OrderPlaced {}
    pub struct BillableOrderPlaced {}
    pub struct OrderAcknowledgmentSent {}

    #[derive(Clone, PartialEq, Eq)]
    pub struct ProductCode(String);

    impl ProductCode {
        pub fn new(code: String) -> Self {
            Self(code)
        }
    }

    pub struct OrderAcknowledgment {}

    pub enum PlaceOrderEvent {
        OrderPlaced(OrderPlaced),
        BillableOrderPlaced(BillableOrderPlaced),
        OrderAcknowledgmentSent(OrderAcknowledgmentSent),
    }

    pub struct OrderId(String);

    impl OrderId {
        pub fn new(id: String) -> Self {
            Self(id)
        }
    }

    pub struct OrderLineId(String);

    impl OrderLineId {
        pub fn new(id: String) -> Self {
            Self(id)
        }
    }
    pub struct OrderQuantity(i64);

    impl OrderQuantity {
        pub fn new(quantity: i64) -> Self {
            Self(quantity)
        }
    }
}

pub mod implementation {

    use crate::{
        public_types::*,
        simple_types::{EmailAddress, String50},
    };
    use anyhow::Result;

    pub struct ValidatedOrderLine {
        order_line_id: OrderLineId,
        product_code: ProductCode,
        quantity: OrderQuantity,
    }

    impl ValidatedOrderLine {
        pub fn new(
            order_line_id: OrderLineId,
            product_code: ProductCode,
            quantity: OrderQuantity,
        ) -> Self {
            Self {
                order_line_id,
                product_code,
                quantity,
            }
        }
    }

    pub struct ValidatedOrder {
        order_id: OrderId,
        // CustomerInfo: CustomerInfo,
        shipping_address: Address,
        billing_address: Address,
        lines: Vec<ValidatedOrderLine>,
    }

    impl ValidatedOrder {
        pub fn new(
            order_id: OrderId,
            shipping_address: Address,
            billing_address: Address,
            lines: Vec<ValidatedOrderLine>,
        ) -> Self {
            Self {
                order_id,
                shipping_address,
                billing_address,
                lines,
            }
        }
    }

    // ======================================================
    // Section 2 : Implementation
    // ======================================================

    // ---------------------------
    // ValidateOrder step
    // ---------------------------

    fn to_customer_info(
        unvalidated_customer_info: UnvalidatedCustomerInfo,
    ) -> Result<ValidatedCustomerInfo> {
        let first_name = String50::create(unvalidated_customer_info.first_name)?;
        let last_name = String50::create(unvalidated_customer_info.last_name)?;
        let email_address = EmailAddress::create(unvalidated_customer_info.email_address)?;
        Ok(ValidatedCustomerInfo::new(
            first_name,
            last_name,
            email_address,
        ))
    }

    fn to_validated_order_line(
        check_product_exists: impl Fn(&ProductCode) -> Result<()>,
        unvalidated_order_line: &UnvalidatedOrderLine,
    ) -> Result<ValidatedOrderLine> {
        let product_code = ProductCode::new(unvalidated_order_line.product_code.clone());
        check_product_exists(&product_code)?;

        let order_line_id = OrderLineId::new(unvalidated_order_line.order_line_id.clone());
        let quantity = OrderQuantity::new(unvalidated_order_line.quantity);

        Ok(ValidatedOrderLine::new(
            order_line_id,
            product_code,
            quantity,
        ))
    }

    fn validate_order(
        check_product_exists: impl Fn(&ProductCode) -> Result<()> + Clone + Copy,
        unvalidated_order: UnvalidatedOrder,
    ) -> Result<ValidatedOrder> {
        let order_id = OrderId::new(unvalidated_order.order_id);
        let shipping_address = Address {};
        let billing_address = Address {};
        let lines = unvalidated_order
            .lines
            .iter()
            .map(|unvalidated_order_line| {
                to_validated_order_line(check_product_exists, unvalidated_order_line)
            })
            .collect::<Result<Vec<ValidatedOrderLine>>>()?;

        Ok(ValidatedOrder::new(
            order_id,
            shipping_address,
            billing_address,
            lines,
        ))
    }

    pub fn place_order(
        check_product_exists: impl Fn(&ProductCode) -> Result<()> + Clone + Copy,
        // check_address_exists: impl Fn(Address) -> Result<()>,
        // get_product_price: impl Fn(ProductCode) -> Result<()>,
        // create_order_acknowledgment_letter: impl Fn(ProductCode) -> Result<OrderAcknowledgment>,
        // send_order_acknowledgment: impl Fn(OrderAcknowledgment) -> Result<()>,
    ) -> impl Fn(UnvalidatedOrder) -> Result<PlaceOrderEvent> {
        move |unvalidated_order| {
            let validated_order = validate_order(check_product_exists, unvalidated_order)?;
            Ok(PlaceOrderEvent::BillableOrderPlaced(BillableOrderPlaced {}))
        }
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Arc;

        use crate::public_types::*;
        use anyhow::anyhow;

        use super::place_order;

        #[test]
        fn it_works() {
            let our_code = ProductCode::new("fake-code".into());
            let product_ids = Arc::new(vec![our_code]);

            let check_product_exists = |code: &ProductCode| {
                if product_ids.contains(&code) {
                    return Ok(());
                }
                return Err(anyhow!("not found!"));
            };
            let unvalidated_order_line = UnvalidatedOrderLine {
                order_line_id: "fake-line-id".into(),
                product_code: "fake-code".into(),
                quantity: 5,
            };
            let unvalidated_order = UnvalidatedOrder {
                order_id: "some_id".into(),
                lines: vec![unvalidated_order_line],
            };

            let workflow = place_order(check_product_exists);
            let result = workflow(unvalidated_order).unwrap();
        }
    }
}
