#!/usr/bin/env python3
"""
Sample Python code with intentional code smells for testing
"""


def process_order(customer_id, product_id, quantity, discount_code,
                  shipping_address, billing_address, payment_method,
                  gift_wrap, special_instructions):
    """
    Process a customer order (Long Method + Long Parameter List smell)
    """
    # Validate customer
    if customer_id is None:
        return None
    if customer_id < 0:
        return None
    if customer_id > 1000000:
        return None

    # Validate product
    if product_id is None:
        return None
    if product_id < 0:
        return None

    # Validate quantity
    if quantity is None:
        return None
    if quantity < 1:
        return None
    if quantity > 100:
        return None

    # Calculate base price
    base_price = 0
    if product_id == 1:
        base_price = 10.00
    elif product_id == 2:
        base_price = 20.00
    elif product_id == 3:
        base_price = 30.00
    elif product_id == 4:
        base_price = 40.00
    elif product_id == 5:
        base_price = 50.00
    else:
        base_price = 100.00

    total = base_price * quantity

    # Apply discount
    if discount_code == "SAVE10":
        total = total * 0.9
    elif discount_code == "SAVE20":
        total = total * 0.8
    elif discount_code == "SAVE30":
        total = total * 0.7
    elif discount_code == "VIP":
        total = total * 0.5

    # Calculate shipping
    shipping_cost = 0
    if shipping_address is not None:
        if "express" in shipping_address.lower():
            shipping_cost = 20.00
        elif "standard" in shipping_address.lower():
            shipping_cost = 5.00
        else:
            shipping_cost = 10.00

    total += shipping_cost

    # Add gift wrap fee
    if gift_wrap:
        total += 5.00

    # Process payment
    if payment_method == "credit_card":
        # Complex payment processing logic
        if total > 1000:
            # High value transaction
            if verify_credit_card():
                charge_credit_card(total)
            else:
                return None
        else:
            charge_credit_card(total)
    elif payment_method == "paypal":
        charge_paypal(total)
    elif payment_method == "bank_transfer":
        initiate_bank_transfer(total)

    # Send confirmation email
    send_email(customer_id, product_id, quantity, total)

    # Update inventory
    update_inventory(product_id, quantity)

    # Log transaction
    log_transaction(customer_id, product_id, total)

    return total


def verify_credit_card():
    """Stub"""
    return True


def charge_credit_card(amount):
    """Stub"""
    pass


def charge_paypal(amount):
    """Stub"""
    pass


def initiate_bank_transfer(amount):
    """Stub"""
    pass


def send_email(customer_id, product_id, quantity, total):
    """Stub"""
    pass


def update_inventory(product_id, quantity):
    """Stub"""
    pass


def log_transaction(customer_id, product_id, total):
    """Stub"""
    pass


def complex_branching(value):
    """
    Feature Envy example - multiple return statements
    """
    if value < 0:
        return "negative"
    elif value == 0:
        return "zero"
    elif value == 1:
        return "one"
    elif value == 2:
        return "two"
    elif value == 3:
        return "three"
    elif value == 4:
        return "four"
    elif value == 5:
        return "five"
    elif value > 100:
        return "large"
    elif value > 50:
        return "medium"
    else:
        return "small"


if __name__ == "__main__":
    result = process_order(
        customer_id=123,
        product_id=1,
        quantity=2,
        discount_code="SAVE10",
        shipping_address="123 Main St (standard)",
        billing_address="123 Main St",
        payment_method="credit_card",
        gift_wrap=True,
        special_instructions="Leave at door"
    )
    print(f"Order total: ${result:.2f}")
