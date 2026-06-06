# Guards against: Data Class FP.
# Ruby Struct generates a value class with accessor methods and equality —
# this is idiomatic Ruby for lightweight data containers, not a smell.

# frozen_string_literal: true

# Immutable user record using Struct.
# Struct provides readers, ==, hash, and to_s — no need for a full class.
User = Struct.new(:id, :name, :email, :role, :created_at, keyword_init: true) do
  # Optional: add a display helper without bloating the struct.
  def display_name
    "#{name} <#{email}>"
  end

  def admin?
    role == "admin"
  end
end

# Product record for an e-commerce catalog.
Product = Struct.new(:sku, :name, :price_cents, :category, :in_stock, keyword_init: true) do
  def price
    format("$%.2f", price_cents / 100.0)
  end

  def available?
    in_stock && price_cents > 0
  end
end

# Event record for audit logging.
AuditEvent = Struct.new(:timestamp, :actor, :action, :resource, :metadata, keyword_init: true) do
  def to_log_line
    "[#{timestamp}] #{actor} #{action} #{resource} #{metadata}"
  end
end
